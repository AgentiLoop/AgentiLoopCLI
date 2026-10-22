#!/usr/bin/env python3
"""Rust ↔ Go parity check for AgentiLoopCLI and AgentiLoopGo.

Runs both binaries against the same fake OpenAI-compatible server (no network,
no API key, a throwaway HOME) and fails if anything the user or the model can
see differs:

  * version, command-line flags and AGENTILOOP_* env vars in --help
  * the /help text and the slash commands the REPL handles
  * every request body sent to the provider (system prompt, tools, history)
  * --json and --stream-json output, the saved session file and settings.json
  * sessions round-trip: Go resumes a Rust session and Rust resumes a Go one
  * the line REPL: /rewind lists prompts, restores their files and cuts the session
  * README headings per language

Usage:
  python3 scripts/parity.py --rust target/debug/agentiloop --go ../AgentiLoopGo/agentiloop \\
      [--rust-src .] [--go-src ../AgentiLoopGo]
"""
import argparse, json, os, re, shutil, socket, subprocess, sys, tempfile, threading
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


class FakeOpenAI(BaseHTTPRequestHandler):
    """First answer of every prompt calls a tool (write_file for prompts starting with "write",
    else list_dir); once a tool result is in, it answers in text."""
    requests = []

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
        self._json(404, {"error": {"message": "not found"}})

    def do_POST(self):
        req = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))) or b"{}")
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

    def __call__(self, *args, check_exit=True, stdin=None):
        FakeOpenAI.requests = []
        p = subprocess.run([self.binary, *args], env=self.env, capture_output=True, text=True, timeout=60,
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
