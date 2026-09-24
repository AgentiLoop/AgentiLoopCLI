# AgentiLoopCLI

🌐 [English](README.md) · [Español](README_es.md) · [Français](README_fr.md) · [Deutsch](README_de.md) · [中文 (简体)](README_zh.md) · [Русский](README_ru.md) · [한국어](README_ko.md) · [日本語](README_ja.md)

### Mac, Windows, Linux용 사전 출시 버전 v0.0.1을 공개했어요!

---

**한번 사용해 보세요!** README를 읽고 AgentiLoop를 실행하기까지 얼마나 걸리는지 확인해 보세요. 문제가 생기면 알려 주세요. 여러분의 피드백을 기다리고 있어요.

**보너스:** Go 버전도 있어요: **AgentiLoopGo** → https://github.com/AgentiLoop/AgentiLoopGo

둘 다 사용해 보고 어느 쪽이 더 나은지 알려 주세요: **Rust일까요, Go일까요?** 🦀 vs 🐹

---

### 💖 AgentiLoop 후원하기

마음에 드셨나요? AgentiLoop가 계속 빠르고 크로스 플랫폼으로 유지될 수 있도록 도와주세요. **[GitHub Sponsors → AgentiLoop](https://github.com/sponsors/AgentiLoop)**에서 후원하실 수 있어요. 후원 등급과 혜택은 [후원 가이드](https://github.com/AgentiLoop/Agent/blob/main/docs/SPONSORSHIP.md)에 있어요.

[![AgentiLoop 후원하기](https://img.shields.io/badge/Sponsor-AgentiLoop-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white)](https://github.com/sponsors/AgentiLoop)

---

AgentiLoop는 Claude Code와 같은 방식으로 터미널에서 실행되는 AI 코딩 에이전트예요. 원하는 것을 평범한 말로 설명하기만 하면 돼요. 에이전트가 파일을 읽고, 코드를 수정하고, 명령어를 실행해서 작업을 끝내요. 무언가를 변경하기 전에는 항상 여러분의 허락을 구해요.

Rust로 작성되었고 macOS, Linux, Windows에서 실행돼요. Claude (Anthropic), OpenAI, Ollama나 LM Studio를 통한 로컬 모델, 그리고 Apple Silicon의 oMLX와 함께 사용할 수 있어요.

AgentiLoop Agent!로 만들었어요. 저희가 정성껏 키운 아이예요. macOS, Linux, Windows용 사전 빌드된 바이너리는 [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases) 페이지에 있고, Rust로 소스에서 직접 컴파일할 수도 있어요.

<img width="2048" height="1152" alt="AgentiLoop가 코딩하는 모습" src="https://github.com/user-attachments/assets/d910bbd2-b47d-4c4a-89af-ac0753ccf279" />

---

## 🚀 처음이신가요? 5분 만에 시작하기

Rust도, Go도, 컴파일도 필요 없어요. 파일 하나를 다운로드하고 API 키를 설정하면 바로 대화를 시작할 수 있어요. 순서대로 따라 해 주세요.

### 1. AgentiLoop 다운로드하기

먼저 어떤 파일이 필요한지 확인하세요:

| 사용 중인 컴퓨터 | 다운로드할 파일 |
|---|---|
| Apple Silicon Mac (M1, M2, M3, M4…) | `agentiloop-macos-arm64.tar.gz` |
| Intel 칩 Mac | `agentiloop-macos-x86_64.tar.gz` |
| Linux, 64비트 PC | `agentiloop-linux-x86_64.tar.gz` |
| ARM 기반 Linux (Raspberry Pi 4/5, ARM 서버) | `agentiloop-linux-arm64.tar.gz` |
| Windows 10/11 | `agentiloop-windows-x86_64.zip` |

잘 모르겠다면 Mac이나 Linux에서 `uname -m`을 실행해 보세요. `arm64` 또는 `aarch64`라면 **arm64**, `x86_64`라면 **x86_64**예요.

**macOS와 Linux.** 터미널을 열고 아래 줄을 붙여 넣으세요. 이 예시는 Apple Silicon용 파일을 사용하므로, 여러분의 환경이 다르다면 처음 세 줄의 `macos-arm64`를 바꿔 주세요:

```sh
curl -LO https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.1/agentiloop-macos-arm64.tar.gz
tar xzf agentiloop-macos-arm64.tar.gz
mkdir -p ~/.local/bin && mv agentiloop-macos-arm64/agentiloop ~/.local/bin/
```

이렇게 하면 프로그램이 홈 디렉터리 안의 폴더인 `~/.local/bin`에 들어가요. 3단계에서 터미널이 그곳을 찾도록 설정해요.

**Windows.** **PowerShell**을 열고 (시작 메뉴 → "PowerShell" 입력) 다음을 붙여 넣으세요:

```powershell
Invoke-WebRequest https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.1/agentiloop-windows-x86_64.zip -OutFile agentiloop.zip
Expand-Archive agentiloop.zip -DestinationPath $HOME\agentiloop -Force
$p = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$p;$HOME\agentiloop\agentiloop-windows-x86_64", "User")
```

마지막 두 줄은 AgentiLoop를 PATH에 추가해요. 변경 사항이 적용되도록 **PowerShell을 닫고 새 창을 여세요**.

### 2. API 키 받기

AgentiLoop는 에이전트예요. "두뇌" 역할은 여기에 연결하는 AI 모델이 해요. **하나**를 선택하세요:

| 선택지 | 받는 곳 | 비용 |
|---|---|---|
| **Claude** (추천) | [console.anthropic.com](https://console.anthropic.com/settings/keys) → *Create Key*. `sk-ant-`로 시작해요 | 사용한 만큼 지불 |
| **OpenAI** | [platform.openai.com/api-keys](https://platform.openai.com/api-keys). `sk-`로 시작해요 | 사용한 만큼 지불 |
| **Ollama** (내 컴퓨터에서 실행) | [ollama.com](https://ollama.com)에서 설치한 다음 `ollama pull qwen2.5-coder`를 실행하세요 | 무료, 키 불필요 |

키를 안전한 곳에 복사해 두세요. 다음 단계에서 붙여 넣을 거예요.

### 3. 셸 프로필에 설정 저장하기

**셸 프로필**은 터미널이 새 창을 열 때마다 읽는 작은 텍스트 파일이에요. 여기에 설정을 넣어 두면 이 작업은 한 번만 하면 돼요. 이 단계를 건너뛰면 새 터미널을 열 때마다 키를 다시 입력해야 해요. 사람들이 막히는 가장 흔한 이유예요.

**어떤 파일인가요?**

| 시스템 | 셸 (기본값) | 프로필 파일 |
|---|---|---|
| macOS (Catalina 10.15 이상) | zsh | `~/.zshrc` |
| 대부분의 Linux 배포판 | bash | `~/.bashrc` |
| zsh를 쓰는 Linux 또는 Mac | zsh | `~/.zshrc` |
| fish 셸 | fish | `~/.config/fish/config.fish` |
| Windows | PowerShell | 필요 없음, 아래 참고 |

어떤 셸을 쓰는지 모르겠다면 `echo $SHELL`을 실행해 보세요. `~`는 홈 폴더를 뜻하므로 `~/.zshrc`는 예를 들어 `/Users/you/.zshrc`예요. 점으로 시작하는 파일은 Finder나 파일 탐색기에서 숨겨져 있는데, 이건 정상이에요.

**파일을 여세요.** 다음 중 하나를 사용하세요 (파일이 아직 없으면 새로 만들어져요):

```sh
nano ~/.zshrc                        # 어디서나 동작해요, 터미널 안에서 바로 편집
touch ~/.zshrc && open -e ~/.zshrc   # macOS: 텍스트 편집기로 열어요
```

(bash를 쓰는 Linux: `~/.zshrc` 대신 `~/.bashrc`를 사용하세요.)

**맨 아래에 다음 줄을 추가하세요.** 필요한 키 줄만 남기고, 따옴표 사이에 실제 키를 붙여 넣으세요:

```sh
# AgentiLoop
export PATH="$HOME/.local/bin:$PATH"

export ANTHROPIC_API_KEY="sk-ant-paste-your-key-here"          # Claude
# export OPENAI_API_KEY="sk-paste-your-key-here"               # OpenAI
# export OPENAI_BASE_URL="http://localhost:11434/v1"           # Ollama (키 불필요)
```

**저장하고 닫으세요.** nano에서는 **Ctrl-O**, **Enter**, 그다음 **Ctrl-X**. 텍스트 편집기에서는 **⌘S**를 누른 뒤 창을 닫으세요.

**불러오세요.** 새 터미널 창을 열거나, 다음을 실행하세요:

```sh
source ~/.zshrc
```

**fish**는 문법이 달라요. 다음 내용을 `~/.config/fish/config.fish`에 넣으세요:

```fish
fish_add_path $HOME/.local/bin
set -gx ANTHROPIC_API_KEY "sk-ant-paste-your-key-here"
```

**Windows (PowerShell).** Windows에는 이 용도로 편집할 프로필 파일이 없어요. 대신 키를 사용자 환경 변수로 저장하세요:

```powershell
setx ANTHROPIC_API_KEY "sk-ant-paste-your-key-here"
# 또는: setx OPENAI_API_KEY "sk-..."   /   setx OPENAI_BASE_URL "http://localhost:11434/v1"
```

그런 다음 **PowerShell을 닫고 새 창을 여세요**. `setx`는 실행한 창 자체에는 적용되지 않아요. 마우스로도 할 수 있어요: 시작 → *Edit environment variables for your account* → *New…*.

> 🔒 **키는 비밀로 지켜 주세요.** 프로필 파일을 git에 커밋하거나 키를 채팅에 붙여 넣지 마세요. Mac에서는 대신 키체인에 보관할 수도 있어요. [빠른 시작의 2단계](#2단계-모델-연결하기)를 참고하세요.

### 4. 잘 동작하는지 확인하기

```sh
agentiloop --version
```

`agentiloop 0.0.1`이 보여야 해요. 이제 키가 불러와졌는지 확인하세요:

```sh
echo $ANTHROPIC_API_KEY | cut -c1-10    # macOS / Linux: sk-ant-...가 출력되어야 해요
```
```powershell
$env:ANTHROPIC_API_KEY.Substring(0,10)  # Windows PowerShell
```

아무것도 출력되지 않으면 3단계로 돌아가세요. 키가 아직 불러와지지 않은 거예요.

### 5. 첫 번째 세션

프로젝트 폴더로 이동해서 전체 화면 인터페이스를 시작하세요:

먼저 새로 만든 빈 테스트 폴더에서 안전하게 사용해 보세요. 실제 프로젝트에서는 대신 `cd`로 해당 프로젝트 폴더로 이동하세요 (예: `cd ~/code/my-app`).

```sh
mkdir -p ~/agentiloop-test
cd ~/agentiloop-test
agentiloop --tui
```

**Ollama**를 쓰시나요? 프로바이더와 pull 받은 모델을 지정하세요: `agentiloop -p openai -m qwen2.5-coder --tui`.

이제 원하는 것을 평범한 말로 입력하고 **Enter**를 누르기만 하면 돼요. 처음 시도해 보기 좋은 프롬프트:

```text
이 프로젝트가 무엇을 하는지 설명해 줘
src 안의 파일을 나열하고 어떤 파일이 진입점인지 알려 줘
TODO 주석을 찾아서 요약해 줘
명령줄 파서에 --verbose 플래그를 추가해 줘
테스트를 실행하고 실패하는 것을 고쳐 줘
이 프로젝트의 README.md를 만들어 줘
```

에이전트는 파일을 변경하거나 명령어를 실행하기 전에 여러분에게 물어봐요. **y**를 누르면 예, **n**을 누르면 아니요, **a**를 누르면 그 세션 동안 해당 도구를 항상 허용, **Esc**를 누르면 그 단계를 건너뛰어요. 종료하려면 **Ctrl-C**를 누르세요. 다음부터는 `agentiloop`만 입력해도 같은 방식으로 시작되고 지난 대화를 이어 가요.

대화 없이 답변 하나만 원하시나요? 질문을 인수로 전달하세요:

```sh
agentiloop "explain what this project does"
```

### 무엇을 할 수 있나요? (도구)

에이전트는 다섯 가지 내장 도구로 작업해요. 여러분이 직접 도구를 호출할 필요는 없어요. 목표를 설명하면 에이전트가 도구를 골라요:

| 도구 | 하는 일 | 먼저 물어보나요? |
|---|---|---|
| `read_file` | 파일을 읽어요 (줄 번호 포함) | 아니요 |
| `list_dir` | 폴더 안의 파일을 나열해요 | 아니요 |
| `write_file` | 새 파일을 만들거나 기존 파일을 덮어써요 | **예** |
| `edit_file` | 파일 안의 특정 텍스트를 정확히 바꿔요 | **예** |
| `bash` | 테스트, 빌드, `git` 같은 셸 명령어를 실행해요 (Mac/Linux에서는 `sh -c`, Windows에서는 `cmd /C`) | **예** |

웹 검색, 데이터베이스, GitHub 같은 도구가 더 필요하신가요? MCP 서버를 추가하세요. [MCP로 도구 추가하기](#mcp로-도구-추가하기-선택-사항)를 참고하세요.

### 도움말 명령어

`agentiloop --help`는 모든 옵션을 보여 줘요:

```text
$ agentiloop --help
AgentiLoop — a cross-platform agentic coding loop for your terminal

Usage: agentiloop [OPTIONS] [PROMPT]...

Arguments:
  [PROMPT]...  One-shot prompt. If omitted, starts an interactive REPL

Options:
  -p, --provider <PROVIDER>      Model backend: `anthropic`, `openai` (OpenAI-compatible: OpenAI, Ollama, LM Studio, Groq, OpenRouter, … via OPENAI_BASE_URL), or `omlx` (local oMLX server, http://localhost:8000/v1). Defaults to the last one used, then auto-detected from which credentials are set [env: AGENTILOOP_PROVIDER=]
  -m, --model <MODEL>            Model id to use. Defaults to the last model used with this provider (~/.agentiloop/settings.json), then the provider's default [env: AGENTILOOP_MODEL=]
      --yes                      Skip all permission prompts (dangerous; intended for CI). Never remembered [env: AGENTILOOP_YES=]
      --max-turns <MAX_TURNS>    Max provider round-trips per prompt [default: last used, then 50]
      --compact-at <COMPACT_AT>  Summarize the conversation once a request reaches this many input tokens (0 = never) [default: last used, then 150000] [env: AGENTILOOP_COMPACT_AT=]
  -C, --cwd <CWD>                Working directory the agent operates in (defaults to cwd)
  -r, --resume <RESUME>          Resume a saved session by id (see /sessions)
  -c, --continue                 Resume the most recent session for this working directory (the default for interactive launches; kept for scripts)
      --new                      Start a new session instead of continuing the last one in this directory
      --tui                      Full-screen terminal UI (ratatui) instead of the line REPL. Remembered [env: AGENTILOOP_TUI=]
      --no-tui                   Use the line REPL even if the TUI was used last time
      --no-mcp                   Don't start MCP servers from ~/.agentiloop/mcp.json / ./.mcp.json [env: AGENTILOOP_NO_MCP=]
  -h, --help                     Print help
  -V, --version                  Print version
```

세션 안에서 `/help`를 입력하면 채팅 명령어 (`/model`, `/sessions`, `/resume`, `/clear`, `/compact`, `/mcp`, `/exit`)를 볼 수 있어요. 전체 설명은 [모든 옵션](#모든-옵션)과 [채팅 안의 명령어](#채팅-안의-명령어)에 있어요.

### 막히셨나요? 빠른 해결 방법

| 보이는 내용 | 해결 방법 |
|---|---|
| `command not found: agentiloop` | `~/.local/bin`이 PATH에 없어요. 3단계의 `export PATH=...` 줄을 추가한 다음 새 터미널을 여세요. Windows에서는 새 PowerShell 창을 여세요 |
| `Error: no provider credentials found` | 불러온 키가 없어요. 3단계를 다시 한 다음 4단계로 확인하세요 |
| macOS: *"agentiloop" cannot be opened* / *unidentified developer* | `curl` 대신 브라우저로 다운로드하면 이런 일이 생겨요. `xattr -d com.apple.quarantine ~/.local/bin/agentiloop`를 실행하세요 |
| Windows: *Windows protected your PC* | **More info** → **Run anyway**를 클릭하세요 |
| `401` / `invalid x-api-key` / 인증 오류 | 키가 틀렸거나 공백 또는 따옴표가 들어 있어요. 다시 복사하고 프로필의 해당 줄을 확인하세요 |
| Ollama: model not found | `ollama list`를 실행하고 정확한 이름을 `-m`으로 전달하세요 |
| 계속 예전 모델이나 프로바이더를 사용해요 | 마지막 선택을 기억하기 때문이에요. `-p` / `-m`을 전달해서 바꾸거나, `~/.agentiloop/settings.json`을 삭제해서 초기화하세요 |

그래도 해결되지 않나요? [이슈를 열고](https://github.com/AgentiLoop/AgentiLoopCLI/issues) 실행한 명령어와 오류를 붙여 넣어 주세요. 저희가 도와드릴게요.

---

## 빠른 시작

세 단계예요: 설치하고, 모델을 연결하고, 실행하세요.

### 1단계: 설치하기

**다운로드:** [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases)에서 여러분의 플랫폼에 맞는 압축 파일을 받아 풀고, `agentiloop` (Windows에서는 `agentiloop.exe`)를 PATH에 넣으세요.

**또는 빌드하기:** 아직 Rust가 없다면 [rustup.rs](https://rustup.rs)에서 설치하세요. 그다음:

```sh
git clone https://github.com/AgentiLoop/AgentiLoopCLI.git
cd AgentiLoopCLI
cargo install --path crates/agentiloop-cli
```

이렇게 하면 프로그램이 빌드되고 `agentiloop` 명령어가 PATH에 있는 `~/.cargo/bin`에 들어가요.

> **설치하지 않으시나요?** 이 README의 모든 내용은 저장소 폴더 안에서도 동작해요. `agentiloop <options>`가 보이는 곳마다 대신 `cargo run -- <options>`를 입력하세요. `--` 뒤의 모든 것은 AgentiLoop에 전달돼요.

### 2단계: 모델 연결하기

AgentiLoop는 대화할 모델이 필요해요. 다음 중 하나를 선택하세요:

| 사용하고 싶은 것… | 할 일 |
|---|---|
| **Claude** (Anthropic) | `export ANTHROPIC_API_KEY=sk-ant-...` |
| **OpenAI** | `export OPENAI_API_KEY=sk-...` |
| **Ollama, LM Studio**, 또는 모든 OpenAI 호환 서버 | `export OPENAI_BASE_URL=http://localhost:11434/v1` (여러분의 서버 주소를 사용하세요. 로컬 서버에는 키가 필요 없어요) |
| **oMLX** (Apple Silicon의 로컬 모델) | 보통은 아무것도 안 해도 돼요. oMLX를 시작한 다음 `-p omlx`로 AgentiLoop를 실행하세요 (아래 참고) |

Claude에는 일반 API 키 (`sk-ant-api…`) 또는 Claude Code 토큰 (`sk-ant-oat01-…`, `claude setup-token`으로 받을 수 있어요)을 사용할 수 있어요. AgentiLoop가 어떤 종류인지 알아서 감지해요.

**oMLX 세부 사항.** oMLX가 같은 Mac에서 실행 중이면, AgentiLoop는 oMLX 자체의 설정 파일 (`~/.omlx/settings.json`)에서 서버 포트와 API 키를 읽어 오므로 아무것도 export할 필요가 없어요. oMLX가 다른 컴퓨터에서 실행 중이거나 그 설정을 덮어쓰고 싶다면 직접 export하세요:

```sh
export OMLX_BASE_URL=http://192.168.1.50:7777/v1   # oMLX 서버 주소 (localhost라면 OMLX_PORT=7777)
export OMLX_API_KEY=...                            # oMLX 설정에 있는 API 키
```

oMLX에서 API 키 검증이 꺼져 있다면 키가 필요 없어요.

`export`는 입력한 터미널 탭에서만 유지돼요. 영구적으로 적용하려면 그 줄을 셸 프로필 (macOS에서는 `~/.zshrc`)에 추가하세요. Mac에서는 키를 파일 대신 키체인에 보관할 수 있어요:

```sh
# 한 번만: 키를 키체인에 저장해요
security add-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w "sk-ant-..."

# ~/.zshrc에 추가: 새 터미널마다 불러와요
export ANTHROPIC_API_KEY="$(security find-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w 2>/dev/null)"
```

### 3단계: 실행하기

작업하려는 프로젝트로 이동해서 AgentiLoop를 시작하세요:

```sh
cd ~/my-project
agentiloop --tui
```

`--tui`는 전체 화면 인터페이스를 열어요. 저희가 추천하는 방식이에요. 원하는 것을 입력하세요. 예를 들어 *"설정 파일을 불러오는 곳을 찾아서 --verbose 플래그를 추가해 줘"*라고 입력하고 Enter를 누르세요.

에이전트의 답변, 사용하는 각 도구 (🔧), 각 결과 (✓ 또는 ✖)가 표시돼요. 아래쪽 상자에는 지금 무엇을 하고 있는지가 표시돼요. 예를 들면 ` ✻ Thinking...  12s `처럼요. 파일을 쓰거나 명령어를 실행하기 전에 여러분에게 물어봐요:

- **y**: 예, 이번만
- **n**: 아니요
- **a**: 남은 세션 동안 이 도구를 항상 허용
- **Esc**: 이 단계는 건너뛰고 계속 진행

---

## 세 가지 사용 방법

| 모드 | 명령어 | 이럴 때 좋아요 |
|---|---|---|
| **TUI** (전체 화면) | `agentiloop --tui` | 일상적인 사용: 기록 스크롤, 실시간 상태, 클릭 가능한 링크 |
| **채팅** (한 줄씩) | `agentiloop` | 단순한 터미널, 또는 일반 텍스트를 선호할 때 |
| **원샷** | `agentiloop "explain this project"` | 질문 하나: 답하고 나서 종료해요. 스크립트에서 편리해요 |

TUI의 키: **Enter** 전송 · **↑ / ↓** 이전 프롬프트 탐색 · **PgUp / PgDn** 또는 마우스 휠로 스크롤 · **Ctrl-U** 줄 지우기 · **Ctrl-C** 종료.

---

## 설정을 기억해요

옵션은 한 번만 입력하면 돼요. AgentiLoop는 실행했던 방식을 저장하므로, 다음부터는 `agentiloop`만 입력해도 같은 방식으로 시작돼요:

```sh
agentiloop -p anthropic --tui    # 처음: 프로바이더와 TUI 선택
agentiloop                       # 이후: 같은 프로바이더, 같은 모델, TUI, 그리고 지난 대화
```

기억하는 것:

- **프로바이더** (`-p`)와 **TUI 켜기/끄기** (`--tui` / `--no-tui`)
- **모델**: 마지막으로 사용한 모델을 프로바이더별로 따로 기억해요. 어떤 프로바이더로 다시 전환하면 그 프로바이더의 모델도 돌아와요.
- **제한 값**: `--max-turns`와 `--compact-at`
- **대화**: 현재 폴더의 마지막 대화가 같은 프로바이더를 사용했다면 그 대화를 이어 가요. 이전 메시지가 화면에 다시 표시되므로, 스크롤해서 어디까지 했는지 확인할 수 있어요

무언가를 바꾸려면 새 옵션을 전달하세요. 바로 적용되고 그때부터 기억돼요:

```sh
agentiloop -p omlx        # oMLX로 전환 (마지막으로 쓴 모델도 돌아와요)
agentiloop -m <model>     # 모델 전환
agentiloop --no-tui       # 한 줄씩 대화하는 채팅으로 돌아가기
agentiloop --new          # 새 대화 시작 (이전 대화는 저장된 채로 남아요)
```

일부 설정은 의도적으로 **절대** 기억하지 않아요:

- `--yes`: 허락 확인을 건너뛰는 것은 매번 신중하게 선택해야 하기 때문이에요
- `--no-mcp`, `-C`, 그리고 원샷 프롬프트
- API 키: 이것들은 셸 프로필에 그대로 두세요

모든 것을 잊게 하려면 `~/.agentiloop/settings.json`을 삭제하세요.

---

## 모든 옵션

모든 옵션은 두 번째 열에 표시된 환경 변수로도 설정할 수 있어요. 직접 입력한 옵션은 기억된 값보다 항상 우선해요.

| 옵션 | 환경 변수 | 하는 일 |
|---|---|---|
| `-p, --provider <name>` | `AGENTILOOP_PROVIDER` | `anthropic`, `openai` 또는 `omlx`. 지정하지 않으면 AgentiLoop는 마지막으로 쓴 것을 사용하거나, 키를 보고 감지해요 (Anthropic, 그다음 OpenAI, 그다음 oMLX 순) |
| `-m, --model <id>` | `AGENTILOOP_MODEL` | 사용할 모델 |
| `--tui` / `--no-tui` | `AGENTILOOP_TUI` | 전체 화면 인터페이스 켜기 / 끄기 |
| `--new` | | 이어 가는 대신 새 대화를 시작해요 |
| `-c, --continue` | | 이 폴더의 마지막 대화를 이어 가요 (이미 기본값이에요) |
| `-r, --resume <id>` | | 특정 대화를 다시 열어요 (id는 `/sessions`로 찾을 수 있어요) |
| `-C, --cwd <folder>` | | 지금 있는 폴더가 아닌 다른 폴더에서 작업해요 |
| `--yes` | `AGENTILOOP_YES` | 도구를 실행하기 전에 묻지 않아요. ⚠️ 신뢰할 수 있는 자동화 용도로만 사용하세요 |
| `--no-mcp` | `AGENTILOOP_NO_MCP` | MCP 서버를 시작하지 않아요 (아래 참고) |
| `--max-turns <n>` | | 요청 하나당 에이전트가 수행할 수 있는 최대 단계 수 (기본값 50) |
| `--compact-at <tokens>` | `AGENTILOOP_COMPACT_AT` | 긴 대화를 요약할 시점 (기본값 150000, `0` = 요약 안 함) |
| `-h` / `-V` | | 도움말 / 버전 |

몇 가지 예시:

```sh
agentiloop -p openai -m gpt-4o-mini "summarize this repo"    # 특정 모델로 질문 하나
agentiloop -C ../other-repo --tui                            # 다른 프로젝트에서 작업
agentiloop --yes "run the tests and fix any failures"        # 무인 실행, 확인 없음
```

**어떤 모델이 사용되나요?** 먼저 해당하는 것이 우선이에요:

1. 명령줄의 `-m`
2. 이어 가는 대화의 모델
3. 이 프로바이더에서 마지막으로 사용한 모델
4. 프로바이더의 기본값: Anthropic은 `claude-sonnet-5`, OpenAI는 `gpt-4o-mini`, oMLX는 제공하는 첫 번째 모델

---

## 채팅 안의 명령어

TUI나 채팅의 프롬프트에서 다음을 입력하세요:

| 명령어 | 하는 일 |
|---|---|
| `/model` | 사용 가능한 모델을 보여 줘요. `/model 3` 또는 `/model <id>`로 전환해요 (기억돼요) |
| `/sessions` | 저장된 대화를 최신순으로 나열해요 |
| `/resume <n or id>` | 그중 하나를 다시 열어요 |
| `/clear` | 대화를 지우고 새 대화를 시작해요 |
| `/compact` | 지금 대화를 요약해서 공간을 확보해요 |
| `/mcp` | 연결된 MCP 서버와 그 도구를 보여 줘요 |
| `/help` | 이 명령어들을 나열해요 |
| `/exit` | 종료해요 |

---

## 긴 대화

모델이 한 번에 읽을 수 있는 양에는 한계가 있어요. 대화가 커지면 (기본적으로 요청이 150,000 토큰에 도달하면), AgentiLoop는 모델에게 지금까지의 내용을 요약하게 하고 그 요약에서부터 이어 가요. 이때 📦 알림이 표시돼요. `/compact`는 원할 때 요약을 실행하고, `--compact-at 0`은 이 기능을 꺼요.

---

## MCP로 도구 추가하기 (선택 사항)

[MCP](https://modelcontextprotocol.io) 서버는 데이터베이스 접근, 웹 검색, 직접 만든 스크립트 같은 추가 도구를 에이전트에게 제공해요. JSON 파일에 목록을 작성하세요:

- `~/.agentiloop/mcp.json`: 모든 프로젝트에서 사용할 수 있어요
- 프로젝트 폴더의 `.mcp.json`: 그 프로젝트에서만 사용할 수 있어요. 두 파일에 같은 이름이 있으면 이쪽이 우선해요.

형식은 Claude Code, Claude Desktop, Agent!에서 사용하는 것과 같으므로 기존 설정을 그대로 복사할 수 있어요:

```json
{ "mcpServers": {
    "Local":  { "command": "my-mcp-server", "args": ["--flag"], "env": { "API_KEY": "${MY_KEY}" } },
    "Remote": { "url": "https://example.com/mcp", "headers": { "Authorization": "Bearer ${TOKEN}" } }
} }
```

동작 방식:

- **두 종류의 서버.** `command`가 있는 서버는 AgentiLoop가 대신 시작해 주는 로컬 프로그램이에요. `url`이 있는 서버는 HTTP로 연결해요. 새로운 "Streamable HTTP" 서버와 예전 "SSE" 서버 모두 동작해요. URL이 `/sse`로 끝나면 (또는 `"transport": "sse"`이면) 예전 방식이 선택돼요.
- **도구 이름.** 각 서버의 도구는 에이전트에게 `mcp_<server>_<tool>` 형태로 보여요. 예: `mcp_Local_search`.
- **비밀 값.** `${VAR}` (또는 `${VAR:-default}`)는 환경 변수에서 채워지므로, 키를 파일에 넣을 필요가 없어요.
- **권한.** MCP 도구도 다른 도구처럼 허락을 구해요. 단, 서버가 해당 도구를 읽기 전용으로 표시한 경우는 예외예요.
- **서버 끄기.** 서버 하나를 건너뛰려면 `"disabled": true`를 추가하고, 모두 건너뛰려면 `--no-mcp`로 실행하세요.
- **안전.** 일반 `http://`는 localhost에서만 허용돼요. 원격 서버에는 `https://`가 필요해요.

`/mcp`를 입력하면 어떤 서버가 연결되었는지, 그 도구와 오류가 있는지 확인할 수 있어요.

---

## 저장 위치

모든 것은 `~/.agentiloop/`에 저장돼요. 다른 폴더를 사용하려면 (예를 들어 별도의 테스트 프로필) `AGENTILOOP_HOME`을 설정하세요.

| 파일 | 내용 |
|---|---|
| `settings.json` | 기억된 프로바이더, 모델, 옵션. 삭제하면 초기화돼요 |
| `sessions/` | 대화 기록, 대화마다 파일 하나 |
| `mcp.json` | MCP 서버 목록 |
| `history.txt` | 입력한 프롬프트 (↑ / ↓ 용) |

---

## 기타 환경 변수

이것들이 필요한 경우는 드물어요:

| 변수 | 용도 |
|---|---|
| `ANTHROPIC_BASE_URL` | Anthropic 요청을 프록시나 호환 서버로 보내요 |
| `ANTHROPIC_OAUTH_TOKEN` | Claude Code 토큰용으로 `ANTHROPIC_API_KEY` 대신 사용해요 |
| `OMLX_BASE_URL`, `OMLX_PORT`, `OMLX_API_KEY` | oMLX 서버 주소와 키. 기본으로 읽는 `~/.omlx/settings.json`보다 우선해요 (둘 다 설정되지 않으면 포트 8000) |
| `RUST_LOG=debug` | 요청별 토큰 사용량을 포함한 디버그 로그를 보여 줘요 |

---

## 개발자를 위한 정보

### 빌드와 테스트

```sh
cargo build --release     # → target/release/agentiloop
cargo test --workspace    # 오프라인으로 실행, API 키 불필요
```

테스트는 네트워크를 사용하지 않아요. 에이전트 루프는 스크립트로 만든 가짜 모델을 상대로, 스트리밍 파서는 로컬 테스트 서버를 상대로 실행돼요. MCP 클라이언트는 함께 제공되는 예제 서버를 사용해 세 가지 연결 방식 (stdio, HTTP, SSE) 모두로 테스트돼요. 이 서버를 직접 실행해서 MCP를 손으로 시험해 볼 수도 있어요:

```sh
cargo run -p agentiloop-mcp --example mcp-example-server -- --http 8791   # 또는 --sse 8792, 또는 --stdio
```

### 코드 구성

프로젝트는 다섯 개의 크레이트로 나뉘어 있고, 각 크레이트는 앞의 크레이트를 기반으로 만들어져요:

| 크레이트 | 내용 |
|---|---|
| `agentiloop-core` | 핵심: 에이전트 루프, 메시지, 도구와 프로바이더 인터페이스, 권한, 세션, 요약 |
| `agentiloop-provider` | 모델과 통신: Anthropic, OpenAI 호환 서버, oMLX |
| `agentiloop-tools` | 내장 도구: `read_file`, `write_file`, `edit_file`, `list_dir`, `bash` |
| `agentiloop-mcp` | MCP 클라이언트, Agent!의 Swift AgentMCP에서 이식했어요 |
| `agentiloop-cli` | `agentiloop` 프로그램: 옵션, 채팅, TUI, 설정 |

### 의존성

의존성은 적게 유지해요: 외부 크레이트는 18개이고, 각 크레이트는 실제로 사용하는 것만 나열해요.

| 크레이트 | 용도 |
|---|---|
| `tokio`, `futures`, `async-trait` | 동시 실행 |
| `serde`, `serde_json` | JSON 읽기와 쓰기 |
| `reqwest` | 모델과 MCP 서버에 대한 HTTP 요청 |
| `anyhow`, `thiserror` | 오류 처리 |
| `tracing`, `tracing-subscriber` | 로깅 |
| `dirs` | 홈 폴더 찾기 |
| `clap` | 명령줄 옵션 |
| `rustyline` | 한 줄씩 대화하는 채팅 |
| `ratatui`, `unicode-width`, `textwrap` | TUI |
| `pulldown-cmark`, `syntect` | Markdown과 코드 하이라이팅 |

---

## 로드맵

- [x] 스트리밍 응답
- [x] OpenAI 호환 프로바이더
- [x] 긴 대화 요약
- [x] 대화 저장
- [x] 전체 화면 TUI
- [x] MCP 클라이언트
- [ ] 다음은 무엇일까요?

## 라이선스

[PolyForm Noncommercial 1.0.0](LICENSE). 이 소프트웨어는 개인적이고 비상업적인 목적으로 사용, 수정, 공유할 수 있어요. 상업용 버전을 만들거나 판매하는 것을 포함한 상업적 사용 권리는 AgentiLoop에 있어요. 상업용 라이선스가 필요하시면 AgentiLoop에 문의하세요.
