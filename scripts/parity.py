#!/usr/bin/env python3
"""Rust ↔ Go parity check for AgentiLoopCLI and AgentiLoopGo.

Runs both binaries against the same fake OpenAI-compatible server (no network,
no API key, a throwaway HOME) and fails if anything the user or the model can
see differs. The same server also plays the Anthropic API for the Claude Code
token checks:

  * version, command-line flags and AGENTILOOP_* env vars in --help
  * the /help text and the slash commands the REPL handles
  * every request body sent to the provider (system prompt, tools, history)
  * --json and --stream-json output, the saved session file and settings.json
  * sessions round-trip: Go resumes a Rust session and Rust resumes a Go one
  * the line REPL: /rewind lists prompts, restores their files and cuts the session; /copy messages;
    /context sizes before and after a prompt
    /memory with no instructions file, then after a # note created AGENTS.md;
    /history lists and searches the saved prompts; /retry undoes the last prompt and sends it again
  * a Claude Code token (sk-ant-oat01-…) on the Anthropic provider: the client profile (headers, beta
    flags, leading system blocks) and what a refused turn does (error, conversation reset, new session)
  * README headings per language

Usage:
  python3 scripts/parity.py --rust target/debug/agentiloop --go ../AgentiLoopGo/agentiloop \\
      [--rust-src .] [--go-src ../AgentiLoopGo]
"""
import argparse, json, os, re, shutil, socket, subprocess, sys, tempfile, threading, time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

FAILS = []


def check(name, rust, go):
    if rust == go:
        print(f"  ok   {name}")
        return
    FAILS.append(name)
    print(f"  FAIL {name}")
    if isinstance(rust, (set, frozenset)):
        print(f"       only Rust: {sorted(rust - go)}\n       only Go:   {sorted(go - rust)}")
    else:
        r, g = json.dumps(rust, indent=1, sort_keys=True), json.dumps(go, indent=1, sort_keys=True)
        import difflib
        for line in list(difflib.unified_diff(r.splitlines(), g.splitlines(), "rust", "go", lineterm=""))[:60]:
            print("       " + line)


# ---- fake provider ----------------------------------------------------------

def chunk(delta, finish=None):
    return {"id": "c1", "object": "chat.completion.chunk", "model": "fake-model",
            "choices": [{"index": 0, "delta": delta, "finish_reason": finish}]}


# What a request made with a Claude Code token must carry (the profile Agent! sends).
CLAUDE_CODE_HEADERS = {
    "authorization": "Bearer sk-ant-oat01-parity",
    "x-api-key": None,
    "user-agent": "claude-cli/2.1.999 (external, cli)",
    "x-app": "cli",
    "accept": "application/json",
    "content-type": "application/json",
    "anthropic-version": "2023-06-01",
    "anthropic-beta": "claude-code-20250219,oauth-2025-04-20,prompt-caching-2024-07-31,extended-cache-ttl-2025-04-11",
    "anthropic-dangerous-direct-browser-access": "true",
}
CLAUDE_CODE_SYSTEM = [
    "x-anthropic-billing-header: cc_version=2.1.999.391; cc_entrypoint=sdk-cli;",
    "You are Claude Code, Anthropic's official CLI for Claude.",
]
REFUSAL = "The model refused this request. The conversation was reset \u2014 rephrase or start a new task."


class FakeOpenAI(BaseHTTPRequestHandler):
    """First answer of every prompt calls a tool (write_file for prompts starting with "write",
    else list_dir); once a tool result is in, it answers in text.

    As the Anthropic API (/v1/messages) it answers "pong", or refuses a prompt starting with "refuse"."""
    requests = []

    def _anthropic(self, req):
        self.requests.append({"POST": self.path, "headers": {h: self.headers.get(h) for h in CLAUDE_CODE_HEADERS}, "body": req})
        last = req["messages"][-1]["content"]
        text = last if isinstance(last, str) else " ".join(b.get("text", "") for b in last)
        events = [{"type": "message_start", "message": {"id": "msg_1", "type": "message", "role": "assistant", "content": [],
                                                        "model": req.get("model"), "usage": {"input_tokens": 10, "output_tokens": 1}}}]
        if not text.startswith("refuse"):
            events += [{"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}},
                       {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "pong"}},
                       {"type": "content_block_stop", "index": 0}]
        stop = "refusal" if text.startswith("refuse") else "end_turn"
        if stop == "refusal":
            time.sleep(1.1)  # session ids are per second: the session that follows a refusal needs a later one
        events += [{"type": "message_delta", "delta": {"stop_reason": stop, "stop_sequence": None}, "usage": {"output_tokens": 5}},
                   {"type": "message_stop"}]
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.end_headers()
        for e in events:
            self.wfile.write(f"event: {e['type']}\ndata: {json.dumps(e)}\n\n".encode())

    def log_message(self, *a):
        pass

    def _json(self, code, obj):
        body = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        self.requests.append({"GET": self.path})
        if self.path.endswith("/models"):
            return self._json(200, {"data": [{"id": "fake-model"}]})
        if "/v1/models/" in self.path:  # Anthropic: one model's limits
            return self._json(200, {"id": self.path.rsplit("/", 1)[1], "display_name": "Fake", "max_input_tokens": 200000, "max_tokens": 8192})
        self._json(404, {"error": {"message": "not found"}})

    def do_POST(self):
        req = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))) or b"{}")
        if self.path.endswith("/v1/messages"):
            return self._anthropic(req)
        self.requests.append({"POST": self.path, "body": req})
        if not self.path.endswith("/chat/completions"):
            return self._json(404, {"error": {"message": "not found"}})
        msgs = req.get("messages", [])
        if msgs and msgs[-1].get("role") == "tool":
            events = [chunk({"role": "assistant", "content": "Done: "}), chunk({"content": "listed."}), chunk({}, "stop")]
        else:
            last = msgs[-1].get("content") if msgs else ""
            last = last if isinstance(last, str) else " ".join(p.get("text", "") for p in last)
            name, args = ("write_file", {"path": "notes.txt", "content": last}) if last.startswith("write") else ("list_dir", {"path": "."})
            events = [chunk({"role": "assistant", "tool_calls": [{"index": 0, "id": "call_1", "type": "function",
                                                                   "function": {"name": name, "arguments": ""}}]}),
                      chunk({"tool_calls": [{"index": 0, "function": {"arguments": json.dumps(args)}}]}),
                      chunk({}, "tool_calls")]
        events.append({"id": "c1", "object": "chat.completion.chunk", "choices": [],
                       "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}})
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.end_headers()
        for e in events:
            self.wfile.write(f"data: {json.dumps(e)}\n\n".encode())
        self.wfile.write(b"data: [DONE]\n\n")


# ---- running the binaries ---------------------------------------------------

class Run:
    def __init__(self, root, name, binary, port):
        self.binary, self.home, self.work = binary, root / f"home-{name}", root / "work"
        self.home.mkdir()
        self.env = {k: v for k, v in os.environ.items() if not k.startswith(("AGENTILOOP_", "ANTHROPIC_", "OPENAI_"))}
        self.env.update(HOME=str(self.home), USERPROFILE=str(self.home), OPENAI_API_KEY="parity",
                        OPENAI_BASE_URL=f"http://127.0.0.1:{port}/v1", AGENTILOOP_YES="1", AGENTILOOP_NO_MCP="1",
                        NO_COLOR="1", TERM="dumb")

    def __call__(self, *args, check_exit=True, stdin=None, env=None):
        FakeOpenAI.requests = []
        p = subprocess.run([self.binary, *args], env={**self.env, **(env or {})}, capture_output=True, text=True, timeout=60,
                           input=stdin, stdin=None if stdin is not None else subprocess.DEVNULL)
        if check_exit and p.returncode != 0:
            raise SystemExit(f"{self.binary} {' '.join(args)} exited {p.returncode}:\n{p.stderr}")
        return p, FakeOpenAI.requests

    def sessions(self):
        return sorted((self.home / ".agentiloop" / "sessions").glob("*.json"))


def norm_session(path):
    s = json.loads(path.read_text())
    for k in ("id", "created", "updated"):
        s.pop(k, None)
    return s


def norm_out(stdout):
    """--json / --stream-json output with the session id and wall-clock timings masked."""
    masked = ("session_id", "elapsed_ms", "first_token_ms")
    return [{**o, **{k: "*" for k in masked if k in o}}
            for o in (json.loads(l) for l in stdout.splitlines() if l.strip())]

def norm_repl(stderr):
    """REPL stderr without blank lines (rustyline and liner space piped input differently) and session ids."""
    return [re.sub(r"\b\d{10}-\d+\b", "<id>", l) for l in stderr.splitlines() if l.strip()]


def repl_rewind(run, work):
    """Two prompts that write notes.txt, then /rewind back through both; returns what to compare."""
    shutil.rmtree(work, ignore_errors=True)
    work.mkdir()
    script = "write first\nwrite second\n/rewind\n/rewind 2\n/rewind\n/rewind 1\n/rewind\n/exit\n"
    p, reqs = run("-p", "openai", "-m", "fake-model", "-C", str(work), "--no-tui", "--new", stdin=script)
    notes = work / "notes.txt"
    return norm_repl(p.stderr), reqs, notes.read_text() if notes.exists() else None, norm_session(run.sessions()[-1])


def repl_copy(run, work):
    """/copy before any reply and with bad arguments (nothing reaches the clipboard); returns the REPL output."""
    p, _ = run("-p", "openai", "-m", "fake-model", "-C", str(work), "--no-tui", "--new", stdin="/copy\n/copy 0\n/copy x\n/copy 2\n/exit\n")
    return norm_repl(p.stderr)


def repl_context(run, work):
    """/context before and after a prompt (same system prompt, tools and history sizes); returns the REPL output."""
    p, _ = run("-p", "openai", "-m", "fake-model", "-C", str(work), "--no-tui", "--new", stdin="/context\nhello\n/context\n/exit\n")
    return norm_repl(p.stderr)
def repl_memory(run, work):
    """/memory before and after a `# note` creates AGENTS.md; returns the REPL output and the file."""
    shutil.rmtree(work, ignore_errors=True)
    work.mkdir()
    p, _ = run("-p", "openai", "-m", "fake-model", "-C", str(work), "--no-tui", "--new", stdin="/memory\n# use tabs\n/memory\n/exit\n")
    return norm_repl(p.stderr), (work / "AGENTS.md").read_text()


def repl_history(run, work):
    """/history with the prompts of the earlier runs, then searches after two more; returns the REPL output."""
    stdin = "/history\nhello\nHello again\n/history HELLO\n/history nope\n/exit\n"
    p, _ = run("-p", "openai", "-m", "fake-model", "-C", str(work), "--no-tui", "--new", stdin=stdin)
    return norm_repl(p.stderr)
def repl_retry(run, work):
    """/retry with nothing to retry, then after a prompt that writes notes.txt; returns what to compare."""
    shutil.rmtree(work, ignore_errors=True)
    work.mkdir()
    p, reqs = run("-p", "openai", "-m", "fake-model", "-C", str(work), "--no-tui", "--new", stdin="/retry\nwrite once\n/retry\n/rewind\n/exit\n")
    notes = work / "notes.txt"
    return norm_repl(p.stderr), reqs, notes.read_text() if notes.exists() else None, norm_session(run.sessions()[-1])


def claude_code(run, work, port):
    """A Claude Code token on the Anthropic provider: a one-shot answer, a one-shot refusal, and a REPL
    where the second prompt is refused; returns what to compare."""
    env = {"ANTHROPIC_API_KEY": "sk-ant-oat01-parity", "ANTHROPIC_BASE_URL": f"http://127.0.0.1:{port}"}
    common = ["-p", "anthropic", "-m", "claude-fable-5-2", "-C", str(work), "--new"]
    p, reqs = run(*common, "--json", "hello", env=env)
    answered = norm_out(p.stdout), reqs
    p, reqs = run(*common, "--stream-json", "refuse this", env=env, check_exit=False)
    refused = p.returncode, norm_out(p.stdout), reqs
    before = len(run.sessions())
    p, reqs = run(*common, "--no-tui", stdin="hello\nrefuse this\nhello again\n/exit\n", env=env)
    replies = [l for l in p.stdout.replace("> ", "").splitlines() if l.strip()]  # the line editors draw the prompt differently
    repl = replies, norm_repl(p.stderr), reqs, [norm_session(s) for s in run.sessions()[before:]]
    return answered, refused, repl


def flags(help_text):
    return set(re.findall(r"(?<![\w-])--?[a-zA-Z][\w-]*", help_text.split("Options:", 1)[-1]))


# ---- source checks ----------------------------------------------------------

def rust_help_text(src):
    body = src.split('const HELP: &str = "', 1)[1].split('";\n', 1)[0]
    return re.sub(r"\\\n\s*", "", body).encode().decode("unicode_escape")


def go_help_text(src):
    body = src.split("const helpText = ", 1)[1].split("\n\n", 1)[0]
    return "".join(json.loads(s) for s in re.findall(r'"(?:[^"\\]|\\.)*"', body))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rust", required=True)
    ap.add_argument("--go", required=True)
    here = Path(__file__).resolve().parent.parent
    ap.add_argument("--rust-src", default=str(here))
    ap.add_argument("--go-src", default=str(here.parent / "AgentiLoopGo"))
    a = ap.parse_args()
    rust_bin, go_bin = str(Path(a.rust).resolve()), str(Path(a.go).resolve())
    rsrc, gsrc = Path(a.rust_src), Path(a.go_src)

    print("source")
    rmain = (rsrc / "crates/agentiloop-cli/src/main.rs").read_text()
    gmain = (gsrc / "cmd/agentiloop/main.go").read_text()
    check("/help text", rust_help_text(rmain), go_help_text(gmain))
    check("slash commands handled",
          set(re.findall(r'"(/[a-z-]+)"(?=[^\n]*=>)', rmain)),
          set(re.findall(r'"(/[a-z-]+)"', "\n".join(l for l in gmain.splitlines() if l.strip().startswith("case \"/")))))
    for suffix in ["", "_de", "_es", "_fr", "_ja", "_ko", "_ru", "_zh"]:
        heads = lambda p: len(re.findall(r"^#{1,6} ", p.read_text(), re.M))
        check(f"README{suffix}.md heading count", heads(rsrc / f"README{suffix}.md"), heads(gsrc / f"README{suffix}.md"))

    server = ThreadingHTTPServer(("127.0.0.1", 0), FakeOpenAI)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    port = server.server_address[1]
    root = Path(tempfile.mkdtemp(prefix="agentiloop-parity-"))
    try:
        (root / "work").mkdir()
        (root / "work" / "a.txt").write_text("hi\n")
        rs, go = Run(root, "rust", rust_bin, port), Run(root, "go", go_bin, port)
        common = ["-p", "openai", "-m", "fake-model", "-C", str(root / "work")]

        print("command line")
        check("--version", rs("--version")[0].stdout.strip(), go("--version")[0].stdout.strip())
        rh, gh = rs("--help")[0].stdout, go("--help")[0].stdout
        check("--help flags", flags(rh), flags(gh))
        check("--help env vars", set(re.findall(r"AGENTILOOP_\w+", rh)), set(re.findall(r"AGENTILOOP_\w+", gh)))

        print("one-shot --json")
        (rp, rreq), (gp, greq) = rs(*common, "--json", "list files"), go(*common, "--json", "list files")
        check("--json stdout", norm_out(rp.stdout), norm_out(gp.stdout))
        check("requests to the provider", rreq, greq)
        check("tool activity on stderr", rp.stderr, gp.stderr)
        check("saved session", norm_session(rs.sessions()[-1]), norm_session(go.sessions()[-1]))
        check("settings.json", json.loads((rs.home / ".agentiloop/settings.json").read_text()),
              json.loads((go.home / ".agentiloop/settings.json").read_text()))

        print("one-shot --stream-json")
        (rp, rreq), (gp, greq) = rs(*common, "--new", "--stream-json", "list files"), go(*common, "--new", "--stream-json", "list files")
        check("--stream-json events", norm_out(rp.stdout), norm_out(gp.stdout))
        check("requests to the provider", rreq, greq)

        print("session round-trip")
        r_id, g_id = rs.sessions()[-1].stem, go.sessions()[-1].stem
        shutil.copy(rs.sessions()[-1], go.home / ".agentiloop/sessions")
        shutil.copy(go.sessions()[-1], rs.home / ".agentiloop/sessions")
        (rp, rreq), (gp, greq) = rs(*common, "-r", g_id, "--json", "again"), go(*common, "-r", r_id, "--json", "again")
        check("resume the other's session: requests", rreq, greq)
        check("resume the other's session: --json stdout", norm_out(rp.stdout), norm_out(gp.stdout))
        check("resume the other's session: saved session",
              norm_session(rs.home / f".agentiloop/sessions/{g_id}.json"),
              norm_session(go.home / f".agentiloop/sessions/{r_id}.json"))

        print("REPL /rewind")
        r, g = repl_rewind(rs, root / "repl"), repl_rewind(go, root / "repl")
        check("/rewind output", r[0], g[0])
        check("/rewind requests to the provider", r[1], g[1])
        check("/rewind leaves notes.txt as before the first prompt", (r[2], g[2]), (None, None))
        check("/rewind saved session", r[3], g[3])

        print("REPL /copy")
        check("/copy output", repl_copy(rs, root / "repl"), repl_copy(go, root / "repl"))
        print("REPL /context")
        check("/context output", repl_context(rs, root / "repl"), repl_context(go, root / "repl"))
        print("REPL /memory")
        r, g = repl_memory(rs, root / "repl"), repl_memory(go, root / "repl")
        check("/memory output", r[0], g[0])
        check("/memory AGENTS.md", r[1], g[1])
        print("REPL /history")
        check("/history output", repl_history(rs, root / "repl"), repl_history(go, root / "repl"))
        print("REPL /retry")
        r, g = repl_retry(rs, root / "repl"), repl_retry(go, root / "repl")
        check("/retry output", r[0], g[0])
        check("/retry requests to the provider", r[1], g[1])
        check("/retry rewrites notes.txt", (r[2], g[2]), ("write once", "write once"))
        check("/retry saved session", r[3], g[3])

        print("Claude Code token (Anthropic)")
        (ra, rr, rl), (ga, gr, gl) = claude_code(rs, root / "repl", port), claude_code(go, root / "repl", port)
        check("--json stdout", ra[0], ga[0])
        check("requests to the provider", ra[1], ga[1])
        sent = lambda reqs: [r for r in reqs if "body" in r]
        want = (CLAUDE_CODE_HEADERS, CLAUDE_CODE_SYSTEM)
        profile = lambda reqs: (sent(reqs)[0]["headers"], [b["text"] for b in sent(reqs)[0]["body"]["system"][:2]])
        check("the Claude Code profile is sent (headers, leading system blocks)", (profile(ra[1]), profile(ga[1])), (want, want))
        check("refused one-shot: exit code and --stream-json events", rr[:2], gr[:2])
        check("refused one-shot: requests to the provider", rr[2], gr[2])
        result = lambda out: (out[-2], out[-1]["is_error"], out[-1].get("error"))
        want = ({"type": "done", "stop_reason": "refusal"}, True, REFUSAL)
        check("refused one-shot: done with stop_reason refusal, then the error", (result(rr[1]), result(gr[1])), (want, want))
        check("REPL refusal: replies on stdout", (rl[0], gl[0]), (["pong", "pong"], ["pong", "pong"]))
        check("REPL refusal: output", rl[1], gl[1])
        check("REPL refusal: requests to the provider", rl[2], gl[2])
        check("REPL refusal: saved sessions", rl[3], gl[3])
        prompts = lambda reqs: [[b["text"] for m in r["body"]["messages"] for b in m["content"]] for r in sent(reqs)]
        want = [["hello"], ["hello", "pong", "refuse this"], ["hello again"]]
        check("REPL refusal: the next prompt starts a new conversation", (prompts(rl[2]), prompts(gl[2])), (want, want))
        titles = lambda sessions: [[b["text"] for m in s["history"] for b in m["content"]] for s in sessions]
        want = [["hello", "pong"], ["hello again", "pong"]]
        check("REPL refusal: the earlier session is kept, the new one holds the rest", (titles(rl[3]), titles(gl[3])), (want, want))
    finally:
        server.shutdown()
        shutil.rmtree(root, ignore_errors=True)

    if FAILS:
        print(f"\n{len(FAILS)} parity check(s) failed: {', '.join(FAILS)}")
        return 1
    print("\nRust and Go are in parity.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
