<a href="https://fluxionai.world/register?source=github&campaign=aiagent&promo=AIAGENT"><img src="docs/sponsors/fluxion-ai-silver-ad_zh.svg" width="900" alt="Fluxion AI，银牌赞助商：一个统一的 API 访问 GPT、Claude 及其他领先的 AI 模型。相比官方 API 价格最高节省 70%，并获得 $3 API 额度。" /></a>

# AgentiLoopCLI

🌐 [English](README.md) · [Español](README_es.md) · [Français](README_fr.md) · [Deutsch](README_de.md) · [中文 (简体)](README_zh.md) · [Русский](README_ru.md) · [한국어](README_ko.md) · [日本語](README_ja.md)

### 🎉 我们发布了适用于 Mac、Windows 和 Linux 的 v0.0.2 正式版本！

---

**快来试试吧！** 阅读这份 README，看看你需要多长时间就能让 AgentiLoop 跑起来。如果遇到任何问题，请告诉我们。我们非常期待你的反馈。

**彩蛋：** 我们还提供了 Go 版本：**AgentiLoopGo** → https://github.com/AgentiLoop/AgentiLoopGo

两个都试一试，然后告诉我们哪个更好用：**Rust 还是 Go？** 🦀 vs 🐹

---

### 💖 赞助 AgentiLoop

喜欢这个项目吗？帮助 AgentiLoop 保持快速和跨平台。欢迎在 **[GitHub Sponsors → AgentiLoop](https://github.com/sponsors/AgentiLoop)** 上赞助我们。赞助档位和回馈请见[赞助指南](https://github.com/AgentiLoop/Agent/blob/main/docs/SPONSORSHIP.md)。

[![赞助 AgentiLoop](https://img.shields.io/badge/Sponsor-AgentiLoop-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white)](https://github.com/sponsors/AgentiLoop)

---

AgentiLoop 是一个在终端中运行的 AI 编程智能体，理念与 Claude Code 相同。你用日常语言描述想要做的事，智能体就会读取你的文件、编辑代码并运行命令来完成任务，而且在做出任何更改之前都会先征得你的同意。

它用 Rust 编写，可在 macOS、Linux 和 Windows 上运行。它支持 Claude（Anthropic）、OpenAI、通过 Ollama 或 LM Studio 运行的本地模型，以及 Apple Silicon 上的 oMLX。

由 AgentiLoop Agent! 创建。这是我们的心血之作。macOS、Linux 和 Windows 的预编译二进制文件可以在 [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases) 页面下载，你也可以用 Rust 从源码编译。

<img width="2048" height="1152" alt="AgentiLoop 编程实况" src="https://github.com/user-attachments/assets/d910bbd2-b47d-4c4a-89af-ac0753ccf279" />

---

## 🚀 第一次使用？5 分钟即可上手

不需要 Rust，不需要 Go，也不需要编译。你只需下载一个文件，提供一个 API 密钥，就可以开始对话了。请按顺序完成以下步骤。

### 1. 下载 AgentiLoop

首先确认你需要哪个文件：

| 你的电脑 | 要下载的文件 |
|---|---|
| 搭载 Apple Silicon（M1、M2、M3、M4…）的 Mac | `agentiloop-macos-arm64.tar.gz` |
| 搭载 Intel 芯片的 Mac | `agentiloop-macos-x86_64.tar.gz` |
| Linux，64 位 PC | `agentiloop-linux-x86_64.tar.gz` |
| ARM 上的 Linux（Raspberry Pi 4/5、ARM 服务器） | `agentiloop-linux-arm64.tar.gz` |
| Windows 10/11 | `agentiloop-windows-x86_64.zip` |

不确定？在 Mac 或 Linux 上运行 `uname -m`。`arm64` 或 `aarch64` 表示 **arm64**，`x86_64` 表示 **x86_64**。

**macOS 和 Linux。** 打开终端并粘贴以下几行。这个示例使用的是 Apple Silicon 版本的文件，如果你的不同，请修改前三行中的 `macos-arm64`：

```sh
curl -LO https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.2/agentiloop-macos-arm64.tar.gz
tar xzf agentiloop-macos-arm64.tar.gz
mkdir -p ~/.local/bin && mv agentiloop-macos-arm64/agentiloop ~/.local/bin/
```

这会把程序放到 `~/.local/bin`，也就是你主目录下的一个文件夹。第 3 步会告诉终端去那里查找它。

**Windows。** 打开 **PowerShell**（开始菜单 → 输入 "PowerShell"）并粘贴：

```powershell
Invoke-WebRequest https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.2/agentiloop-windows-x86_64.zip -OutFile agentiloop.zip
Expand-Archive agentiloop.zip -DestinationPath $HOME\agentiloop -Force
$p = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$p;$HOME\agentiloop\agentiloop-windows-x86_64", "User")
```

最后两行会把 AgentiLoop 添加到你的 PATH 中。**关闭 PowerShell 并打开一个新窗口**，让更改生效。

### 2. 获取 API 密钥

AgentiLoop 是智能体，而它的"大脑"是你为它连接的 AI 模型。请选择**一个**：

| 选项 | 获取方式 | 费用 |
|---|---|---|
| **Claude**（推荐） | [console.anthropic.com](https://console.anthropic.com/settings/keys) → *Create Key*。密钥以 `sk-ant-` 开头 | 按用量付费 |
| **OpenAI** | [platform.openai.com/api-keys](https://platform.openai.com/api-keys)。密钥以 `sk-` 开头 | 按用量付费 |
| **Ollama**（在你自己的电脑上运行） | 从 [ollama.com](https://ollama.com) 安装，然后运行 `ollama pull qwen2.5-coder` | 免费，无需密钥 |

把密钥复制到一个安全的地方。下一步会用到它。

### 3. 在 shell 配置文件中保存设置

你的 **shell 配置文件**是一个小文本文件，终端每次打开新窗口时都会读取它。把设置写在那里，你只需要做一次。如果跳过这一步，你就得在每个新终端里重新输入密钥。这是大家卡住的头号原因。

**是哪个文件？**

| 系统 | Shell（默认） | 配置文件 |
|---|---|---|
| macOS（Catalina 10.15 及更高版本） | zsh | `~/.zshrc` |
| 大多数 Linux 发行版 | bash | `~/.bashrc` |
| 使用 zsh 的 Linux 或 Mac | zsh | `~/.zshrc` |
| fish shell | fish | `~/.config/fish/config.fish` |
| Windows | PowerShell | 不需要，见下文 |

不确定自己用的是哪个 shell？运行 `echo $SHELL`。`~` 表示你的主文件夹，所以 `~/.zshrc` 例如就是 `/Users/you/.zshrc`。以点开头的文件在 Finder 和文件浏览器中是隐藏的，这很正常。

**打开文件。** 使用下面任意一种方式（如果文件还不存在，它们会创建该文件）：

```sh
nano ~/.zshrc                        # 在任何地方都能用，直接在终端里编辑
touch ~/.zshrc && open -e ~/.zshrc   # macOS：在 TextEdit 中打开
```

（使用 bash 的 Linux：用 `~/.bashrc` 代替 `~/.zshrc`。）

**在文件末尾添加以下几行。** 只保留你需要的那行密钥，并把你真实的密钥粘贴到引号之间：

```sh
# AgentiLoop
export PATH="$HOME/.local/bin:$PATH"

export ANTHROPIC_API_KEY="sk-ant-paste-your-key-here"          # Claude
# export OPENAI_API_KEY="sk-paste-your-key-here"               # OpenAI
# export OPENAI_BASE_URL="http://localhost:11434/v1"           # Ollama（无需密钥）
```

**保存并关闭。** 在 nano 中：按 **Ctrl-O**、**Enter**，然后按 **Ctrl-X**。在 TextEdit 中：按 **⌘S**，然后关闭窗口。

**加载设置。** 打开一个新的终端窗口，或者运行：

```sh
source ~/.zshrc
```

**fish** 使用不同的语法。把下面的内容放进 `~/.config/fish/config.fish`：

```fish
fish_add_path $HOME/.local/bin
set -gx ANTHROPIC_API_KEY "sk-ant-paste-your-key-here"
```

**Windows（PowerShell）。** Windows 没有需要为此编辑的配置文件。请改为把密钥保存为用户环境变量：

```powershell
setx ANTHROPIC_API_KEY "sk-ant-paste-your-key-here"
# 或者：setx OPENAI_API_KEY "sk-..."   /   setx OPENAI_BASE_URL "http://localhost:11434/v1"
```

然后**关闭 PowerShell 并打开一个新窗口**。`setx` 不会影响运行它的那个窗口。你也可以用鼠标完成：开始 → *Edit environment variables for your account* → *New…*。

> 🔒 **请保管好你的密钥。** 不要把配置文件提交到 git，也不要把密钥粘贴到聊天中。在 Mac 上，你也可以把它存放在钥匙串中；请参阅[快速入门的第 2 步](#第-2-步连接模型)。

### 4. 检查是否正常工作

```sh
agentiloop --version
```

你应该会看到 `agentiloop 0.0.2`。接下来检查密钥是否已加载：

```sh
echo $ANTHROPIC_API_KEY | cut -c1-10    # macOS / Linux：应输出 sk-ant-...
```
```powershell
$env:ANTHROPIC_API_KEY.Substring(0,10)  # Windows PowerShell
```

如果什么都没有输出，请回到第 3 步。密钥还没有加载。

### 5. 你的第一次会话

进入一个项目文件夹，启动全屏界面：

先新建一个空的测试文件夹，放心地试一试。如果要用于真实项目，请改用 `cd` 进入该项目的文件夹（例如 `cd ~/code/my-app`）。

```sh
mkdir -p ~/agentiloop-test
cd ~/agentiloop-test
agentiloop --tui
```

使用 **Ollama**？告诉它提供方以及你已拉取的模型：`agentiloop -p openai -m qwen2.5-coder --tui`。

现在只需用日常语言输入你想做的事，然后按 **Enter**。下面是一些不错的入门提示：

```text
解释这个项目是做什么的
列出 src 中的文件，并告诉我哪个是入口文件
找出 TODO 注释并总结它们
给命令行解析器添加一个 --verbose 参数
运行测试并修复所有失败的地方
为这个项目创建一个 README.md
```

在智能体修改文件或运行命令之前，它会先询问你。按 **y** 表示同意，按 **n** 表示拒绝，按 **a** 表示在本次会话中始终允许该工具，按 **Esc** 跳过这一步。按 **Ctrl-C** 退出。下次只需输入 `agentiloop`，它就会以同样的方式启动，并接着你上次的对话继续。

只想得到一个答案，而不进入聊天？把问题作为参数传入：

```sh
agentiloop "explain what this project does"
```

### 它能做什么？（工具）

智能体使用五个内置工具。你不需要自己调用它们。你只需描述目标，智能体会自己选择工具：

| 工具 | 功能 | 是否先询问？ |
|---|---|---|
| `read_file` | 读取文件（带行号） | 否 |
| `list_dir` | 列出文件夹中的文件 | 否 |
| `write_file` | 创建新文件或覆盖已有文件 | **是** |
| `edit_file` | 修改文件中一段精确的文本 | **是** |
| `bash` | 运行 shell 命令，例如测试、构建或 `git`（Mac/Linux 上为 `sh -c`，Windows 上为 `cmd /C`） | **是** |

想要更多工具，比如网页搜索、数据库或 GitHub？添加 MCP 服务器即可；请参阅[通过 MCP 添加工具](#通过-mcp-添加工具可选)。

### 帮助命令

`agentiloop --help` 会列出所有选项：

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

在会话中输入 `/help` 可查看聊天命令（`/model`、`/sessions`、`/resume`、`/clear`、`/compact`、`/mcp`、`/exit`）。完整参考请见[全部选项](#全部选项)和[聊天中的命令](#聊天中的命令)。

### 卡住了？快速解决

| 你看到的 | 解决方法 |
|---|---|
| `command not found: agentiloop` | `~/.local/bin` 不在你的 PATH 中。添加第 3 步中的 `export PATH=...` 那一行，然后打开一个新终端。在 Windows 上，打开一个新的 PowerShell 窗口 |
| `Error: no provider credentials found` | 没有加载任何密钥。重新完成第 3 步，然后按第 4 步检查 |
| macOS：*"agentiloop" cannot be opened* / *unidentified developer* | 如果你是用浏览器而不是 `curl` 下载的，就会出现这种情况。运行 `xattr -d com.apple.quarantine ~/.local/bin/agentiloop` |
| Windows：*Windows protected your PC* | 点击 **More info** → **Run anyway** |
| `401` / `invalid x-api-key` / 身份验证错误 | 密钥不正确，或者其中含有空格或引号。重新复制密钥，并检查配置文件中的那一行 |
| Ollama：model not found | 运行 `ollama list`，然后用 `-m` 传入准确的名称 |
| 它一直在使用旧的模型或提供方 | 它会记住你上次的选择。传入 `-p` / `-m` 来更改，或删除 `~/.agentiloop/settings.json` 来重置 |

还是卡住了？[提交一个 issue](https://github.com/AgentiLoop/AgentiLoopCLI/issues)，并贴上你运行的命令和错误信息。我们会帮助你。

---

## 快速入门

三个步骤：安装、连接模型、运行。

### 第 1 步：安装

**下载：** 从 [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases) 获取适合你平台的压缩包，解压后把 `agentiloop`（Windows 上为 `agentiloop.exe`）放到你的 PATH 中。

**或者自行构建：** 如果你还没有安装 Rust，请从 [rustup.rs](https://rustup.rs) 安装。然后：

```sh
git clone https://github.com/AgentiLoop/AgentiLoopCLI.git
cd AgentiLoopCLI
cargo install --path crates/agentiloop-cli
```

这会构建程序，并在你的 PATH（`~/.cargo/bin`）中放入一个 `agentiloop` 命令。

> **不想安装？** 本 README 中的所有内容也可以在仓库文件夹内直接使用。凡是看到 `agentiloop <options>` 的地方，改为输入 `cargo run -- <options>` 即可。`--` 之后的所有内容都会传给 AgentiLoop。

### 第 2 步：连接模型

AgentiLoop 需要一个可以对话的模型。请从下面选择一个：

| 我想使用… | 这样做 |
|---|---|
| **Claude**（Anthropic） | `export ANTHROPIC_API_KEY=sk-ant-...` |
| **OpenAI** | `export OPENAI_API_KEY=sk-...` |
| **Ollama、LM Studio** 或任何兼容 OpenAI 的服务器 | `export OPENAI_BASE_URL=http://localhost:11434/v1`（使用你的服务器地址；本地服务器无需密钥） |
| **oMLX**（Apple Silicon 上的本地模型） | 通常什么都不用做。启动 oMLX，然后用 `-p omlx` 运行 AgentiLoop（见下文） |

对于 Claude，你可以使用普通的 API 密钥（`sk-ant-api…`），也可以使用 Claude Code 令牌（`sk-ant-oat01-…`，可通过 `claude setup-token` 获取）。AgentiLoop 会自动识别是哪一种。

**oMLX 详情。** 当 oMLX 运行在同一台 Mac 上时，AgentiLoop 会从 oMLX 自己的设置文件（`~/.omlx/settings.json`）中读取服务器端口和 API 密钥，所以你不需要导出任何变量。如果 oMLX 运行在另一台机器上，或者你想覆盖这些设置，请自行导出：

```sh
export OMLX_BASE_URL=http://192.168.1.50:7777/v1   # oMLX 服务器的地址（本机可用 OMLX_PORT=7777）
export OMLX_API_KEY=...                            # oMLX 设置中的 API 密钥
```

如果 oMLX 关闭了 API 密钥验证，则不需要密钥。

`export` 只在你输入它的那个终端标签页中有效。要让它永久生效，请把这一行添加到你的 shell 配置文件中（macOS 上为 `~/.zshrc`）。在 Mac 上，你可以把密钥存放在钥匙串中，而不是写在文件里：

```sh
# 只需一次：把密钥存入钥匙串
security add-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w "sk-ant-..."

# 写在 ~/.zshrc 中：每个新终端都会加载它
export ANTHROPIC_API_KEY="$(security find-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w 2>/dev/null)"
```

### 第 3 步：运行

进入你要处理的项目，启动 AgentiLoop：

```sh
cd ~/my-project
agentiloop --tui
```

`--tui` 会打开全屏界面，我们推荐使用它。输入你想做的事，例如 *"找到加载配置文件的位置，并添加一个 --verbose 参数"*，然后按 Enter。

你会看到智能体的回复、它使用的每个工具（🔧）以及每个结果（✓ 或 ✖）。底部的框会显示它当前正在做什么，例如 ` ✻ Thinking...  12s `。在写入文件或运行命令之前，它会询问你：

- **y**：同意，仅这一次
- **n**：拒绝
- **a**：在本次会话剩余时间内始终允许该工具
- **Esc**：跳过这一步，但继续进行

---

## 三种使用方式

| 模式 | 命令 | 适用场景 |
|---|---|---|
| **TUI**（全屏） | `agentiloop --tui` | 日常使用：可滚动的历史记录、实时状态、可点击的链接 |
| **聊天**（逐行） | `agentiloop` | 简单的终端，或者你更喜欢纯文本 |
| **单次** | `agentiloop "explain this project"` | 只问一个问题：回答后就退出。在脚本中很方便 |

TUI 中的按键：**Enter** 发送 · **↑ / ↓** 浏览之前的提示 · **PgUp / PgDn** 或鼠标滚轮滚动 · **Ctrl-U** 清空当前行 · **Ctrl-C** 退出。

---

## 它会记住你的设置

你只需输入一次选项。AgentiLoop 会保存你的启动方式，所以下次只需输入 `agentiloop` 就会以同样的方式启动：

```sh
agentiloop -p anthropic --tui    # 第一次：选择提供方和 TUI
agentiloop                       # 之后：相同的提供方、相同的模型、TUI，以及你上次的对话
```

它会记住：

- **提供方**（`-p`）和 **TUI 开/关**（`--tui` / `--no-tui`）
- **模型**：你上次使用的模型，每个提供方分别记录。切换回某个提供方时，它的模型也会恢复。
- **限制**：`--max-turns` 和 `--compact-at`
- **你的对话**：如果当前文件夹中的上一次对话使用的是同一个提供方，它会接着那次对话继续。之前的消息会重新显示在屏幕上，你可以向上滚动，看看上次进行到哪里

要更改某项设置，传入新的选项即可。它会立即生效，并从此被记住：

```sh
agentiloop -p omlx        # 切换到 oMLX（它上次使用的模型也会恢复）
agentiloop -m <model>     # 切换模型
agentiloop --no-tui       # 回到逐行聊天
agentiloop --new          # 开始一段新对话（旧对话仍会保存）
```

有些内容是特意**从不**记住的：

- `--yes`：跳过权限询问必须每次都是有意识的选择
- `--no-mcp`、`-C` 和单次提示
- API 密钥：它们保留在你的 shell 配置文件中

要清除所有记忆，请删除 `~/.agentiloop/settings.json`。

---

## 全部选项

每个选项也都可以通过环境变量设置，见第二列。你输入的选项总是优先于记住的值。

| 选项 | 环境变量 | 功能 |
|---|---|---|
| `-p, --provider <name>` | `AGENTILOOP_PROVIDER` | `anthropic`、`openai` 或 `omlx`。如果你没有指定，AgentiLoop 会使用上次的提供方，或根据你的密钥自动检测（先 Anthropic，然后 OpenAI，最后 oMLX） |
| `-m, --model <id>` | `AGENTILOOP_MODEL` | 要使用的模型 |
| `--tui` / `--no-tui` | `AGENTILOOP_TUI` | 开启 / 关闭全屏界面 |
| `--new` | | 开始新对话，而不是继续之前的对话 |
| `-c, --continue` | | 继续此处的上一次对话（已是默认行为） |
| `-r, --resume <id>` | | 重新打开某个特定对话（用 `/sessions` 查找 id） |
| `-C, --cwd <folder>` | | 在与当前所在位置不同的文件夹中工作 |
| `--yes` | `AGENTILOOP_YES` | 运行工具前不询问。⚠️ 仅用于可信的自动化场景 |
| `--no-mcp` | `AGENTILOOP_NO_MCP` | 不启动 MCP 服务器（见下文） |
| `--max-turns <n>` | | 智能体每个请求最多可执行的步数（默认 50） |
| `--compact-at <tokens>` | `AGENTILOOP_COMPACT_AT` | 何时对长对话进行总结（默认 150000，`0` = 从不） |
| `-h` / `-V` | | 帮助 / 版本 |

一些示例：

```sh
agentiloop -p openai -m gpt-4o-mini "summarize this repo"    # 用指定模型问一个问题
agentiloop -C ../other-repo --tui                            # 处理另一个项目
agentiloop --yes "run the tests and fix any failures"        # 无人值守，不询问
```

**使用哪个模型？** 按以下顺序，第一个适用的生效：

1. 命令行上的 `-m`
2. 你正在继续的对话所用的模型
3. 你上次在此提供方上使用的模型
4. 提供方的默认模型：Anthropic 为 `claude-sonnet-5`，OpenAI 为 `gpt-4o-mini`，oMLX 则为它提供的第一个模型

---

## 聊天中的命令

在 TUI 或聊天的提示符处输入这些命令：

| 命令 | 功能 |
|---|---|
| `/model` | 显示可用模型。`/model 3` 或 `/model <id>` 可切换模型（并会被记住） |
| `/sessions` | 列出你保存的对话，最新的在前 |
| `/resume <n or id>` | 重新打开其中一个对话 |
| `/clear` | 清除当前对话并开始新对话 |
| `/compact` | 立即总结对话以释放空间 |
| `/mcp` | 显示已连接的 MCP 服务器及其工具 |
| `/help` | 列出这些命令 |
| `/exit` | 退出 |

---

## 长对话

模型一次能读取的内容是有限的。当对话变得很长时（默认是请求达到 150,000 个 token 时），AgentiLoop 会让模型总结目前为止的内容，并在总结的基础上继续。发生这种情况时，你会看到一条 📦 提示。`/compact` 可以按需执行总结，`--compact-at 0` 则会关闭此功能。

---

## 通过 MCP 添加工具（可选）

[MCP](https://modelcontextprotocol.io) 服务器可以为智能体提供额外的工具，例如数据库访问、网页搜索或你自己的脚本。在 JSON 文件中列出它们：

- `~/.agentiloop/mcp.json`：在所有项目中可用
- 项目文件夹中的 `.mcp.json`：仅在该项目中可用。如果同一个名称出现在两个文件中，以这个文件为准。

格式与 Claude Code、Claude Desktop 和 Agent! 使用的相同，因此你可以直接复制现有配置：

```json
{ "mcpServers": {
    "Local":  { "command": "my-mcp-server", "args": ["--flag"], "env": { "API_KEY": "${MY_KEY}" } },
    "Remote": { "url": "https://example.com/mcp", "headers": { "Authorization": "Bearer ${TOKEN}" } }
} }
```

工作原理：

- **两种服务器。** 带有 `command` 的服务器是 AgentiLoop 为你启动的本地程序。带有 `url` 的服务器通过 HTTP 访问。较新的 "Streamable HTTP" 服务器和较旧的 "SSE" 服务器都可以使用；以 `/sse` 结尾的 URL（或 `"transport": "sse"`）会选择旧的方式。
- **工具名称。** 每个服务器工具在智能体中显示为 `mcp_<server>_<tool>`，例如 `mcp_Local_search`。
- **机密信息。** `${VAR}`（或 `${VAR:-default}`）会从你的环境变量中填入，因此密钥不需要写在文件里。
- **权限。** MCP 工具和其他工具一样会请求权限，除非服务器将某个工具标记为只读。
- **关闭服务器。** 添加 `"disabled": true` 可跳过某个服务器，或者使用 `--no-mcp` 运行以跳过全部服务器。
- **安全。** 普通的 `http://` 只允许用于 localhost；远程服务器需要使用 `https://`。

输入 `/mcp` 可查看哪些服务器已连接、它们的工具以及任何错误。

---

## 文件保存位置

所有内容都保存在 `~/.agentiloop/` 中。设置 `AGENTILOOP_HOME` 可以使用其他文件夹，例如单独的测试配置。

| 文件 | 内容 |
|---|---|
| `settings.json` | 记住的提供方、模型和选项。删除它即可重置 |
| `sessions/` | 你的对话，每个对话一个文件 |
| `mcp.json` | 你的 MCP 服务器 |
| `history.txt` | 你输入过的提示（用于 ↑ / ↓） |

---

## 其他环境变量

你很少会需要这些：

| 变量 | 用途 |
|---|---|
| `ANTHROPIC_BASE_URL` | 将 Anthropic 请求发送到代理或兼容服务器 |
| `ANTHROPIC_OAUTH_TOKEN` | 使用 Claude Code 令牌时，可替代 `ANTHROPIC_API_KEY` |
| `OMLX_BASE_URL`, `OMLX_PORT`, `OMLX_API_KEY` | oMLX 服务器地址和密钥。它们会覆盖默认读取的 `~/.omlx/settings.json`（如果两者都未设置，则使用端口 8000） |
| `RUST_LOG=debug` | 显示调试日志，包括每个请求的 token 用量 |

---

## 开发者指南

### 构建与测试

```sh
cargo build --release     # → target/release/agentiloop
cargo test --workspace    # 离线运行，无需 API 密钥
```

测试不会访问网络。智能体循环针对一个脚本化的模拟模型运行，流式解析器则针对本地测试服务器运行。MCP 客户端通过全部三种连接类型（stdio、HTTP、SSE）针对一个内置的示例服务器进行测试。你也可以自己运行这个服务器，手动试用 MCP：

```sh
cargo run -p agentiloop-mcp --example mcp-example-server -- --http 8791   # 或 --sse 8792，或 --stdio
```

### 代码结构

项目分为五个 crate，每一个都建立在前面几个的基础之上：

| Crate | 内容 |
|---|---|
| `agentiloop-core` | 核心：智能体循环、消息、工具和提供方接口、权限、会话、总结 |
| `agentiloop-provider` | 与模型通信：Anthropic、兼容 OpenAI 的服务器、oMLX |
| `agentiloop-tools` | 内置工具：`read_file`、`write_file`、`edit_file`、`list_dir`、`bash` |
| `agentiloop-mcp` | MCP 客户端，移植自 Agent! 的 Swift AgentMCP |
| `agentiloop-cli` | `agentiloop` 程序：选项、聊天、TUI、设置 |

### 依赖

我们尽量保持依赖精简：共 18 个外部 crate，并且每个 crate 只列出它实际用到的依赖。

| Crate | 用途 |
|---|---|
| `tokio`, `futures`, `async-trait` | 并发执行 |
| `serde`, `serde_json` | 读写 JSON |
| `reqwest` | 向模型和 MCP 服务器发送 HTTP 请求 |
| `anyhow`, `thiserror` | 错误处理 |
| `tracing`, `tracing-subscriber` | 日志 |
| `dirs` | 查找你的主文件夹 |
| `clap` | 命令行选项 |
| `rustyline` | 逐行聊天 |
| `ratatui`, `unicode-width`, `textwrap` | TUI |
| `pulldown-cmark`, `syntect` | Markdown 和代码高亮 |

---

## 路线图

- [x] 流式响应
- [x] 兼容 OpenAI 的提供方
- [x] 总结长对话
- [x] 保存对话
- [x] 全屏 TUI
- [x] MCP 客户端
- [ ] 下一步是什么？

## 许可证

[PolyForm Noncommercial 1.0.0](LICENSE)。你可以出于个人和非商业目的使用、修改和分享本软件。商业用途（包括构建或销售商业版本）仅限 AgentiLoop 保留。如需商业许可，请联系 AgentiLoop。

---

<a href="https://fluxionai.world/register?source=github&campaign=aiagent&promo=AIAGENT"><img src="docs/sponsors/fluxion-ai-silver-ad_zh.svg" width="900" alt="Fluxion AI，银牌赞助商：一个统一的 API 访问 GPT、Claude 及其他领先的 AI 模型。相比官方 API 价格最高节省 70%，并获得 $3 API 额度。" /></a>
