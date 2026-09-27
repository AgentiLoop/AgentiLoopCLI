<a href="https://fluxionai.world/register?source=github&campaign=aiagent&promo=AIAGENT"><img src="docs/sponsors/fluxion-ai-silver-ad_ja.svg" width="900" alt="Fluxion AI（シルバースポンサー）：GPT、Claude などの主要 AI モデルをひとつの統合 API で。公式 API 価格と比べて最大 70% お得、さらに $3 分の API クレジット。" /></a>

# AgentiLoopCLI

🌐 [English](README.md) · [Español](README_es.md) · [Français](README_fr.md) · [Deutsch](README_de.md) · [中文 (简体)](README_zh.md) · [Русский](README_ru.md) · [한국어](README_ko.md) · [日本語](README_ja.md)

### 🎉 Mac、Windows、Linux 向けのリリース版 v0.0.2 を公開しました！

---

**ぜひ試してみてください！** README を読んで、AgentiLoop を動かすまでにどれくらい時間がかかるか確かめてみてください。問題があればお知らせください。皆さんのフィードバックをお待ちしています。

**おまけ:** Go 版もあります: **AgentiLoopGo** → https://github.com/AgentiLoop/AgentiLoopGo

両方試して、どちらが優れているか教えてください: **Rust か Go か？** 🦀 vs 🐹

---

### 💖 AgentiLoop をスポンサーする

気に入っていただけましたか？ AgentiLoop を高速かつクロスプラットフォームに保つためにご支援ください。**[GitHub Sponsors → AgentiLoop](https://github.com/sponsors/AgentiLoop)** でスポンサーになれます。プランと特典は[スポンサーシップガイド](https://github.com/AgentiLoop/Agent/blob/main/docs/SPONSORSHIP.md)に載っています。

[![AgentiLoop をスポンサーする](https://img.shields.io/badge/Sponsor-AgentiLoop-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white)](https://github.com/sponsors/AgentiLoop)

---

AgentiLoop は、Claude Code と同じ発想でターミナル上で動く AI コーディングエージェントです。やりたいことを普通の言葉で伝えるだけです。エージェントがファイルを読み、コードを編集し、コマンドを実行して作業を進めます。何かを変更する前には、必ずあなたの許可を求めます。

Rust で書かれていて、macOS、Linux、Windows で動作します。Claude (Anthropic)、OpenAI、Ollama や LM Studio 経由のローカルモデル、そして Apple Silicon 上の oMLX に対応しています。

AgentiLoop Agent! で作られました。私たちの自慢の子です。macOS、Linux、Windows 向けのビルド済みバイナリは [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases) ページにあります。Rust を使ってソースからコンパイルすることもできます。

<img width="2048" height="1152" alt="AgentiLoop がコーディングしている様子" src="https://github.com/user-attachments/assets/d910bbd2-b47d-4c4a-89af-ac0753ccf279" />

---

## ⚡ 初回起動: セットアップウィザードがすべてやってくれます

手動で設定するものは何もありません。`agentiloop` を初めて実行すると、組み込みのセットアップウィザードが自動的に起動します。使いたいプロバイダー (Claude、OpenAI、Ollama / LM Studio、または oMLX) を尋ね、API キーを受け取り (入力は隠されます)、キーが動くことを確認し、モデルを選ばせてから、すべてを `~/.agentiloop` に保存します。所要時間は 1 分ほどで、設定ファイルも `export` 行も不要です。

<img src="docs/setup-wizard-tui.png" width="900" alt="フルスクリーン TUI 内で動くセットアップウィザード: プロバイダー、隠された API キー、接続確認、モデル一覧、キーの保存先、そして最初のプロンプト" />

`agentiloop --setup` でいつでも再実行できます (フルスクリーン版にするには `--tui` を追加するか、セッション中に `/setup` と入力します)。`agentiloop --reset` はすべてを忘れて初期状態からやり直します。コマンドラインのプログラムをインストールしたことがありませんか？ 下の[はじめての方へ](#-はじめての方へ-5-分で使い始めましょう)を、順番どおりに進めてください。

---

## 🚀 はじめての方へ: 5 分で使い始めましょう

Rust も Go もコンパイルも不要です。ファイルを 1 つダウンロードし、API キーを設定すれば、すぐにチャットを始められます。手順どおりに進めてください。

### 1. AgentiLoop をダウンロードする

まず、どのファイルが必要かを確認します:

| お使いのコンピューター | ダウンロードするファイル |
|---|---|
| Apple Silicon 搭載の Mac (M1、M2、M3、M4…) | `agentiloop-macos-arm64.tar.gz` |
| Intel チップ搭載の Mac | `agentiloop-macos-x86_64.tar.gz` |
| Linux、64 ビット PC | `agentiloop-linux-x86_64.tar.gz` |
| ARM 上の Linux (Raspberry Pi 4/5、ARM サーバー) | `agentiloop-linux-arm64.tar.gz` |
| Windows 10/11 | `agentiloop-windows-x86_64.zip` |

わからない場合は、Mac または Linux で `uname -m` を実行してください。`arm64` または `aarch64` なら **arm64**、`x86_64` なら **x86_64** です。

**macOS と Linux。** ターミナルを開いて、次の行を貼り付けます。この例は Apple Silicon 用のファイルを使っているので、お使いの環境が違う場合は最初の 3 行の `macos-arm64` を書き換えてください:

```sh
curl -LO https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.2/agentiloop-macos-arm64.tar.gz
tar xzf agentiloop-macos-arm64.tar.gz
mkdir -p ~/.local/bin && mv agentiloop-macos-arm64/agentiloop ~/.local/bin/
```

これでプログラムがホームディレクトリ内のフォルダー `~/.local/bin` に置かれます。ステップ 3 のセットアップウィザードが、ターミナルがそこを探すように設定することを提案してくれます。

**Windows。** **PowerShell** を開き (スタートメニュー → 「PowerShell」と入力)、次を貼り付けます:

```powershell
Invoke-WebRequest https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.2/agentiloop-windows-x86_64.zip -OutFile agentiloop.zip
Expand-Archive agentiloop.zip -DestinationPath $HOME\agentiloop -Force
$p = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$p;$HOME\agentiloop\agentiloop-windows-x86_64", "User")
```

最後の 2 行で AgentiLoop を PATH に追加します。変更を反映させるため、**PowerShell を閉じて新しいウィンドウを開いてください**。

### 2. API キーを取得する

AgentiLoop はエージェントです。「頭脳」となるのは、接続する AI モデルです。**1 つ**選んでください:

| 選択肢 | 入手先 | 料金 |
|---|---|---|
| **Claude** (おすすめ) | [console.anthropic.com](https://console.anthropic.com/settings/keys) → *Create Key*。`sk-ant-` で始まります | 従量課金 |
| **OpenAI** | [platform.openai.com/api-keys](https://platform.openai.com/api-keys)。`sk-` で始まります | 従量課金 |
| **Ollama** (自分のコンピューターで動作) | [ollama.com](https://ollama.com) からインストールし、`ollama pull qwen2.5-coder` を実行します | 無料、キー不要 |

キーを安全な場所にコピーしておきましょう。次のステップで貼り付けます。

### 3. セットアップウィザードを実行する

ファイルを編集したり、`export` コマンドを入力したりする必要はありません。AgentiLoop には組み込みのセットアップウィザードがあり、いくつか質問したうえでキーを保存してくれます。

`~/.local/bin` はまだ PATH に入っていないので、今回だけはフルパスで起動します (Windows ではステップ 1 で PATH がすでに設定済みなので、`agentiloop` と入力するだけです):

```sh
~/.local/bin/agentiloop
```

ウィザードは 5 つの短いステップを案内します。**Enter** を押すとデフォルトを受け入れ、番号を入力すると選択できます:

1. **プロバイダー**: Claude なら `1`、OpenAI なら `2`、Ollama なら `3` を入力します。
2. **API キー**: ステップ 2 のキーを貼り付けて Enter を押します。貼り付けても画面には何も表示されませんが、これは意図的なもので、キーが隠されています。Ollama にはキーが不要です: サーバーアドレスで Enter、キーでもう一度 Enter を押してください。
3. **接続確認**: ウィザードはすぐにキーを試します。失敗した場合は理由を表示し、再入力させてくれます。うまくいくまで何も保存されません。
4. **モデル**: Enter を押してデフォルトにするか、一覧から番号を入力します。
5. **キーの保存先**: Enter を押します。これでキーは `~/.agentiloop/env` という、AgentiLoop だけが読むプライベートなファイルに保存されます。

最後に、ウィザードは `~/.local/bin` が PATH に入っていないことに気づき、`Add it to PATH in /Users/you/.zshrc?` と尋ねます。**Enter** (はい) を押してください。すると `All set` と表示され、プロンプトに入ります。

Claude を選んでデフォルトを受け入れた場合、全体の流れはこのようになります:

```text
$ ~/.local/bin/agentiloop
Welcome to AgentiLoop! Let's set things up (about a minute).
Settings are kept in /Users/you/.agentiloop. Run `agentiloop --setup` or `/setup` to redo this, `agentiloop --reset` to start over.

Which model provider do you want to use?
  1  Claude (Anthropic) — API key from console.anthropic.com
  2  OpenAI — API key from platform.openai.com
  3  Ollama, LM Studio or another OpenAI-compatible server (local, usually no key)
  4  oMLX (local Apple Silicon server; reads ~/.omlx/settings.json)
Provider [1-4, default 1]: 1
Anthropic API key (starts with sk-ant-, input hidden):
Checking the connection…
Connected (8 model(s) available).

Pick a model (change it any time with /model):
   1  claude-sonnet-5  (default)
   2  claude-…
   3  claude-…
   …
Model [1-8, an id, or Enter for claude-sonnet-5]:

Where should the credential be saved?
  1  /Users/you/.agentiloop/env (recommended; only agentiloop reads it, file mode 600)
  2  Also add it to /Users/you/.zshrc so other tools in your terminal see it
  3  macOS Keychain, with a line in /Users/you/.zshrc that reads it (nothing stored in plain text)
Save to [1-3, default 1]:
saved to /Users/you/.agentiloop/env

`/Users/you/.local/bin` is not on your PATH, so `agentiloop` only works with its full path.
Add it to PATH in /Users/you/.zshrc? [Y/n]
updated /Users/you/.zshrc (between `# >>> agentiloop >>>` and `# <<< agentiloop <<<`); it applies to new terminals

All set: anthropic / claude-sonnet-5. Type a request at the prompt, /help for commands, /exit to leave.
```

ここでそのままチャットを始めてもいいですし、`/exit` と入力してステップ 4 に進んでもかまいません。間違えましたか？ `agentiloop --setup` でウィザードをもう一度実行でき、`agentiloop --reset` で保存した内容をすべて消去できます。

> 🔒 **キーは秘密にしてください。** `~/.agentiloop/env` はあなただけが読めるファイルです。キーをチャットに貼り付けたり、git にコミットしたりしないでください。Mac では、最後の質問で 3 を選ぶと代わりにキーチェーンに保存されるので、平文でディスクに置かれることはありません。

環境変数を使ってキーを自分で管理したいですか？ それも可能ですが、上級者向けの方法です。クイックスタートの[上級者向け: キーを手動で設定する](#ステップ-2-モデルを接続する)を参照してください。

### 4. 動作を確認する

ウィザードによる PATH の変更を反映させるため、**新しいターミナルウィンドウを開いてください** (Windows では新しい PowerShell ウィンドウ)。そのうえで:

```sh
agentiloop --version
```

`agentiloop 0.0.2` と表示されるはずです。次に、オプションなしで実行します:

```sh
agentiloop
```

そのままプロンプトが開くはずです。代わりにウィザードがもう一度起動した場合は、キーが保存されていません。ステップ 3 をもう一度やり直してください。

### 5. 最初のセッション

プロジェクトフォルダーに移動して、フルスクリーンのインターフェースを起動します:

まずは新しい空のテスト用フォルダーで、安心して試してみましょう。実際のプロジェクトで使うときは、代わりに `cd` でそのプロジェクトのフォルダーに移動してください（例：`cd ~/code/my-app`）。

```sh
mkdir -p ~/agentiloop-test
cd ~/agentiloop-test
agentiloop --tui
```

**Ollama** を使っていますか？ ウィザードがサーバーと選んだモデルをすでに記憶しています。pull 済みの別のモデルに切り替えるには、セッション中に `/model` と入力してください。

あとは、やりたいことを普通の言葉で入力して **Enter** を押すだけです。最初に試すのにおすすめのプロンプト:

```text
このプロジェクトが何をするのか説明して
src 内のファイルを一覧にして、どれがエントリーポイントか教えて
TODO コメントを探して要約して
コマンドラインパーサーに --verbose フラグを追加して
テストを実行して、失敗したものを修正して
このプロジェクトの README.md を作成して
```

エージェントはファイルを変更したりコマンドを実行したりする前に、あなたに確認します。**y** で許可、**n** で拒否、**a** でそのセッション中はそのツールを常に許可、**Esc** でそのステップをスキップします。終了するには **Ctrl-C** を押します。次回からは、`agentiloop` とだけ入力すれば同じ設定で起動し、前回の会話の続きから始まります。

チャットなしで答えを 1 つだけ知りたいですか？ 質問を引数として渡してください:

```sh
agentiloop "explain what this project does"
```

### 何ができますか？ (ツール)

エージェントは 5 つの組み込みツールを使って作業します。自分でツールを呼び出す必要はありません。目的を伝えれば、エージェントがツールを選びます:

| ツール | 機能 | 事前に確認？ |
|---|---|---|
| `read_file` | ファイルを読み込みます (行番号付き) | いいえ |
| `list_dir` | フォルダー内のファイルを一覧表示します | いいえ |
| `write_file` | 新しいファイルを作成するか、既存のファイルを上書きします | **はい** |
| `edit_file` | ファイル内の特定のテキストを正確に書き換えます | **はい** |
| `bash` | テスト、ビルド、`git` などのシェルコマンドを実行します (Mac/Linux では `sh -c`、Windows では `cmd /C`) | **はい** |

Web 検索、データベース、GitHub など、もっとツールが欲しいですか？ MCP サーバーを追加してください。[MCP でツールを追加する](#mcp-でツールを追加する任意)を参照してください。

### ヘルプコマンド

`agentiloop --help` ですべてのオプションが表示されます:

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

セッション中に `/help` と入力すると、チャットコマンド (`/model`、`/sessions`、`/resume`、`/clear`、`/compact`、`/mcp`、`/exit`) が表示されます。詳しいリファレンスは[すべてのオプション](#すべてのオプション)と[チャット内のコマンド](#チャット内のコマンド)にあります。

### うまくいかないときは？ すぐできる解決策

| 表示される内容 | 解決策 |
|---|---|
| `command not found: agentiloop` | `~/.local/bin` が PATH に入っていません。まず新しいターミナルウィンドウを開いてください。それでも直らなければ `~/.local/bin/agentiloop --setup` を実行し、PATH に追加するか聞かれたら「はい」と答えてください。Windows では、新しい PowerShell ウィンドウを開いてください |
| `Error: no provider credentials found` | キーが保存されていません。`agentiloop --setup` を実行し (ステップ 3)、ステップ 4 で確認してください |
| macOS: *"agentiloop" cannot be opened* / *unidentified developer* | `curl` ではなくブラウザーでダウンロードした場合に起こります。`xattr -d com.apple.quarantine ~/.local/bin/agentiloop` を実行してください |
| Windows: *Windows protected your PC* | **More info** → **Run anyway** をクリックしてください |
| `401` / `invalid x-api-key` / 認証エラー | キーが間違っているか、スペースや引用符が混ざった状態で貼り付けられています。もう一度コピーして、`agentiloop --setup` を実行して入力し直してください |
| Ollama: model not found | `ollama list` を実行して、正確な名前を `-m` で渡してください |
| 古いモデルやプロバイダーが使われ続ける | 前回の選択を記憶しているためです。`-p` / `-m` を渡して変更するか、`agentiloop --reset` を実行して最初からやり直してください |

それでも解決しない場合は、[Issue を作成](https://github.com/AgentiLoop/AgentiLoopCLI/issues)して、実行したコマンドとエラーを貼り付けてください。私たちがお手伝いします。

---

## クイックスタート

3 つのステップです: インストールして、モデルを設定して、実行します。

### ステップ 1: インストール

**ダウンロード:** [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases) からお使いのプラットフォーム用のアーカイブを入手して展開し、`agentiloop` (Windows では `agentiloop.exe`) を PATH の通った場所に置きます。

**またはビルド:** Rust をまだ持っていない場合は、[rustup.rs](https://rustup.rs) からインストールしてください。その後:

```sh
git clone https://github.com/AgentiLoop/AgentiLoopCLI.git
cd AgentiLoopCLI
cargo install --path crates/agentiloop-cli
```

これでプログラムがビルドされ、`agentiloop` コマンドが PATH 上の `~/.cargo/bin` に配置されます。

> **インストールしない場合は？** この README の内容は、すべてリポジトリのフォルダー内からでも使えます。`agentiloop <options>` と書かれている箇所では、代わりに `cargo run -- <options>` と入力してください。`--` より後ろはすべて AgentiLoop に渡されます。

### ステップ 2: モデルを接続する

AgentiLoop には、対話するためのモデルが必要です。**そのために環境変数を設定する必要はありません**: 組み込みのセットアップウィザードがいくつか質問して、すべてを保存してくれます。

#### セットアップウィザード (おすすめ)

`agentiloop` を実行するだけです。初回、まだキーが設定されていないときは、ウィザードが自動的に起動します。所要時間は 1 分ほどで、次の 5 つを尋ねます:

1. **どのプロバイダーか** — Claude、OpenAI、ローカルの OpenAI 互換サーバー (Ollama、LM Studio、…)、または oMLX。番号を入力します。
2. **API キー** — 入力は隠され、画面には何も表示されません。ローカルサーバーでは通常不要です。同じ Mac 上の oMLX の場合、ウィザードは oMLX 自身の設定からキーを読み込むので、そもそも尋ねられません。
3. **接続確認** — ウィザードはその場でプロバイダーと通信します。キーが間違っていればそう伝えて、再入力を促します。うまくいくまで何も保存されません。
4. **どのモデルか** — プロバイダーが返した一覧から 1 つ選ぶか、Enter を押してデフォルトにします。後から `/model` でいつでも変更できます。
5. **キーの保存先** — Enter を押してデフォルトの `~/.agentiloop/env` にします。これは AgentiLoop だけが読むプライベートなファイルです。(キーをシェルや macOS のキーチェーンにも置きたい人向けの他の選択肢は、下の*上級者向け*で説明します。)

その後 `All set` と表示され、プロンプトに入ります。通常のターミナルで Claude を選び、デフォルトを受け入れた場合の一連の流れはこうなります (モデルの一覧はお使いの環境によって異なります):

```text
$ agentiloop
Welcome to AgentiLoop! Let's set things up (about a minute).
Settings are kept in /Users/you/.agentiloop. Run `agentiloop --setup` or `/setup` to redo this, `agentiloop --reset` to start over.

Which model provider do you want to use?
  1  Claude (Anthropic) — API key from console.anthropic.com
  2  OpenAI — API key from platform.openai.com
  3  Ollama, LM Studio or another OpenAI-compatible server (local, usually no key)
  4  oMLX (local Apple Silicon server; reads ~/.omlx/settings.json)
Provider [1-4, default 1]: 1
Anthropic API key (starts with sk-ant-, input hidden):
Checking the connection…
Connected (8 model(s) available).

Pick a model (change it any time with /model):
   1  claude-sonnet-5  (default)
   2  claude-…
   3  claude-…
   …
Model [1-8, an id, or Enter for claude-sonnet-5]:

Where should the credential be saved?
  1  /Users/you/.agentiloop/env (recommended; only agentiloop reads it, file mode 600)
  2  Also add it to /Users/you/.zshrc so other tools in your terminal see it
  3  macOS Keychain, with a line in /Users/you/.zshrc that reads it (nothing stored in plain text)
Save to [1-3, default 1]:
saved to /Users/you/.agentiloop/env

All set: anthropic / claude-sonnet-5. Type a request at the prompt, /help for commands, /exit to leave.
```

これで完了です。以降は `agentiloop` を実行すると、そのままプロンプトが開きます。

Claude では、通常の API キー (`sk-ant-api…`) と Claude Code のトークン (`sk-ant-oat01-…`、`claude setup-token` で取得できます) のどちらでも貼り付けられます。AgentiLoop がどちらの種類かを自動で判別します。3 (Ollama、LM Studio、…) を選んだ場合、ウィザードはサーバーの URL も尋ね、デフォルトとして `http://localhost:11434/v1` を提示するので、ローカルの Ollama なら Enter を押すだけです。

**同じウィザードをフルスクリーンで。** ウィザードは使っているインターフェースの中で動きます。`--tui` を付けて起動すると、同じ質問がフルスクリーン画面の中に表示され、終わるとそのままプロンプトに入っています:

<img src="docs/setup-wizard-tui.png" width="900" alt="フルスクリーン TUI 内で動くセットアップウィザード: プロバイダー、隠された API キー、接続確認、モデル一覧、キーの保存先、そして最初のプロンプト" />

**いつでも再実行・やり直しできます:**

```bash
agentiloop --setup          # 通常のターミナルでウィザード
agentiloop --setup --tui    # フルスクリーン TUI の中でウィザード (スクリーンショットのように)
/setup                      # 実行中のセッションの中から (REPL でも TUI でも)
agentiloop --reset          # すべてを忘れて初期状態からやり直す
```

<details>
<summary><b>上級者向け: キーを手動で設定する</b> (ウィザードでうまくいった場合はスキップしてください)</summary>

キーを自分で管理したい場合や、誰もウィザードに答えられないスクリプトや CI で AgentiLoop を動かす場合は、次の環境変数のいずれかを設定してください。AgentiLoop は何も尋ねずにそれを使います:

| 使いたいもの… | 設定するもの |
|---|---|
| **Claude** (Anthropic) | `export ANTHROPIC_API_KEY=sk-ant-...` |
| **OpenAI** | `export OPENAI_API_KEY=sk-...` |
| **Ollama、LM Studio**、または任意の OpenAI 互換サーバー | `export OPENAI_BASE_URL=http://localhost:11434/v1` (お使いのサーバーのアドレス。ローカルサーバーならキーは不要です) |
| **oMLX** (Apple Silicon 上のローカルモデル) | 通常は何もしなくて大丈夫です。oMLX を起動してから、`-p omlx` を付けて AgentiLoop を実行します |

**oMLX の詳細。** oMLX が同じ Mac で動いている場合、AgentiLoop は oMLX 自身の設定ファイル (`~/.omlx/settings.json`) からサーバーのポートと API キーを読み込みます。oMLX が別のマシンで動いている場合や、その設定を上書きしたい場合は:

```sh
export OMLX_BASE_URL=http://192.168.1.50:7777/v1   # oMLX サーバーのアドレス (localhost なら OMLX_PORT=7777)
export OMLX_API_KEY=...                            # oMLX の設定にある API キー
```

oMLX で API キーの検証がオフになっている場合、キーは不要です。

`export` は、入力したターミナルのタブでしか有効になりません。永続的にするには、その行をシェルプロファイル (macOS では `~/.zshrc`) に追加することになりますが、これはまさにウィザードの **「Also add it to ~/.zshrc」** の選択肢がやってくれることです。同様に、ウィザードの **「macOS Keychain」** の選択肢は、次の手順を自動でやってくれるものです:

```sh
# 一度だけ: キーをキーチェーンに保存します
security add-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w "sk-ant-..."

# ~/.zshrc に記述: 新しいターミナルを開くたびに読み込みます
export ANTHROPIC_API_KEY="$(security find-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w 2>/dev/null)"
```

</details>

### ステップ 3: 実行する

作業したいプロジェクトに移動して、AgentiLoop を起動します:

```sh
cd ~/my-project
agentiloop --tui
```

`--tui` はフルスクリーンのインターフェースを開きます。こちらがおすすめです。やりたいこと、例えば *「設定ファイルを読み込んでいる場所を探して、--verbose フラグを追加して」* と入力し、Enter を押します。

エージェントの返答、使った各ツール (🔧)、各結果 (✓ または ✖) が表示されます。下部のボックスには、今何をしているかが表示されます。例えば ` ✻ Thinking...  12s ` のようにです。ファイルを書き込んだりコマンドを実行したりする前に、あなたに確認します:

- **y**: はい、今回は許可
- **n**: いいえ
- **a**: このセッションの残りの間、このツールを常に許可
- **Esc**: このステップはスキップして、作業は続行

---

## 3 つの使い方

| モード | コマンド | 向いている用途 |
|---|---|---|
| **TUI** (フルスクリーン) | `agentiloop --tui` | 普段使い: 履歴のスクロール、リアルタイムのステータス表示、クリックできるリンク |
| **チャット** (1 行ずつ) | `agentiloop` | シンプルなターミナルや、プレーンテキストが好みの場合 |
| **ワンショット** | `agentiloop "explain this project"` | 1 つの質問: 回答したら終了します。スクリプトで便利です |

TUI のキー操作: **Enter** で送信 · **↑ / ↓** で以前のプロンプトをたどる · **PgUp / PgDn** またはマウスホイールでスクロール · **Ctrl-U** で行をクリア · **Ctrl-C** で終了。

---

## 設定を記憶します

オプションを入力するのは一度だけです。AgentiLoop は起動時の設定を保存するので、次回からは `agentiloop` とだけ入力すれば同じ設定で起動します:

```sh
agentiloop -p anthropic --tui    # 初回: プロバイダーと TUI を選択
agentiloop                       # 次回以降: 同じプロバイダー、同じモデル、TUI、そして前回の会話
```

記憶される内容:

- **プロバイダー** (`-p`) と **TUI のオン/オフ** (`--tui` / `--no-tui`)
- **モデル**: 最後に使ったモデルを、プロバイダーごとに別々に記憶します。プロバイダーを切り替えて戻ると、そのプロバイダーのモデルも戻ります。
- **上限値**: `--max-turns` と `--compact-at`
- **会話**: 現在のフォルダーでの前回の会話が同じプロバイダーを使っていた場合、その続きから始まります。以前のメッセージが画面に再表示されるので、スクロールして前回どこまで進めたかを確認できます

何かを変更したいときは、新しいオプションを渡してください。すぐに反映され、それ以降も記憶されます:

```sh
agentiloop -p omlx        # oMLX に切り替え (最後に使ったモデルも戻ります)
agentiloop -m <model>     # モデルを切り替え
agentiloop --no-tui       # 1 行ずつのチャットに戻す
agentiloop --new          # 新しい会話を開始 (以前の会話は保存されたまま)
```

いくつかの設定は、意図的に**決して**記憶されません:

- `--yes`: 許可の確認をスキップするのは、毎回意識して選ぶべきことだからです
- `--no-mcp`、`-C`、ワンショットのプロンプト
- API キー: これらは `~/.agentiloop/env` (ウィザードが書き込みます) またはシェルの環境変数に置かれ、`settings.json` には決して保存されません

すべてを忘れさせるには、`agentiloop --reset` を実行してください。

---

## すべてのオプション

すべてのオプションは、2 列目に示した環境変数でも設定できます。入力したオプションは、記憶されている値より常に優先されます。

| オプション | 環境変数 | 機能 |
|---|---|---|
| `-p, --provider <name>` | `AGENTILOOP_PROVIDER` | `anthropic`、`openai`、または `omlx`。指定しない場合、AgentiLoop は前回のものを使うか、キーから自動検出します (Anthropic、次に OpenAI、次に oMLX の順) |
| `-m, --model <id>` | `AGENTILOOP_MODEL` | 使用するモデル |
| `--tui` / `--no-tui` | `AGENTILOOP_TUI` | フルスクリーンのインターフェースのオン / オフ |
| `--new` | | 続きからではなく、新しい会話を開始します |
| `-c, --continue` | | このフォルダーでの前回の会話を続けます (すでにデフォルトの動作です) |
| `-r, --resume <id>` | | 特定の会話を再開します (id は `/sessions` で確認できます) |
| `-C, --cwd <folder>` | | 今いるフォルダーとは別のフォルダーで作業します |
| `--yes` | `AGENTILOOP_YES` | ツールを実行する前に確認しません。⚠️ 信頼できる自動化用途でのみ使ってください |
| `--no-mcp` | `AGENTILOOP_NO_MCP` | MCP サーバーを起動しません (下記参照) |
| `--setup` | | 初回セットアップウィザードを再実行します (プロバイダー、キー、モデル)。`--tui` と組み合わせるとフルスクリーン画面の中で実行されます |
| `--reset` | | 初期状態に戻します: `~/.agentiloop`、シェルプロファイル内の agentiloop ブロック、ウィザードが作成したキーチェーン項目（Windows では、ウィザードが設定したユーザー環境変数）を削除します。手書きの `export` 行はコメントアウトされるだけで、しかも同意した場合のみです。`--yes` を付けると確認を省略します |
| `--max-turns <n>` | | 1 回のリクエストでエージェントが実行できる最大ステップ数 (デフォルト 50) |
| `--compact-at <tokens>` | `AGENTILOOP_COMPACT_AT` | 長い会話を要約するタイミング (デフォルト 150000、`0` = しない) |
| `-h` / `-V` | | ヘルプ / バージョン |

いくつかの例:

```sh
agentiloop -p openai -m gpt-4o-mini "summarize this repo"    # 特定のモデルで質問を 1 つ
agentiloop -C ../other-repo --tui                            # 別のプロジェクトで作業
agentiloop --yes "run the tests and fix any failures"        # 無人実行、確認なし
```

**どのモデルが使われますか？** 最初に当てはまるものが優先されます:

1. コマンドラインの `-m`
2. 続きから再開する会話のモデル
3. このプロバイダーで最後に使ったモデル
4. プロバイダーのデフォルト: Anthropic なら `claude-sonnet-5`、OpenAI なら `gpt-4o-mini`、oMLX なら提供される最初のモデル

---

## チャット内のコマンド

TUI またはチャットのプロンプトで、次のコマンドを入力します:

| コマンド | 機能 |
|---|---|
| `/model` | 利用可能なモデルを表示します。`/model 3` または `/model <id>` で切り替えます (記憶されます) |
| `/sessions` | 保存された会話を新しい順に一覧表示します |
| `/resume <n or id>` | そのうちの 1 つを再開します |
| `/clear` | 会話をクリアして新しい会話を始めます |
| `/compact` | 今すぐ会話を要約して容量を空けます |
| `/mcp` | 接続中の MCP サーバーとそのツールを表示します |
| `/help` | これらのコマンドを一覧表示します |
| `/exit` | 終了します |

---

## 長い会話

モデルが一度に読める量には限りがあります。会話が大きくなると (デフォルトでは、リクエストが 150,000 トークンに達したとき)、AgentiLoop はモデルにそれまでの内容を要約させ、その要約から作業を続けます。そのときは 📦 のメッセージが表示されます。`/compact` で任意のタイミングで要約でき、`--compact-at 0` でこの機能をオフにできます。

---

## MCP でツールを追加する（任意）

[MCP](https://modelcontextprotocol.io) サーバーを使うと、データベースへのアクセス、Web 検索、自作スクリプトなど、エージェントに追加のツールを与えられます。JSON ファイルに一覧を書きます:

- `~/.agentiloop/mcp.json`: すべてのプロジェクトで使えます
- プロジェクトフォルダー内の `.mcp.json`: そのプロジェクトでのみ使えます。両方のファイルに同じ名前がある場合は、こちらが優先されます。

形式は Claude Code、Claude Desktop、Agent! と同じなので、既存の設定をそのままコピーできます:

```json
{ "mcpServers": {
    "Local":  { "command": "my-mcp-server", "args": ["--flag"], "env": { "API_KEY": "${MY_KEY}" } },
    "Remote": { "url": "https://example.com/mcp", "headers": { "Authorization": "Bearer ${TOKEN}" } }
} }
```

仕組み:

- **2 種類のサーバー。** `command` を持つサーバーは、AgentiLoop が代わりに起動するローカルプログラムです。`url` を持つサーバーには HTTP で接続します。新しい「Streamable HTTP」と古い「SSE」のどちらのサーバーにも対応しています。URL が `/sse` で終わる場合 (または `"transport": "sse"` の場合) は古い方式が選ばれます。
- **ツール名。** 各サーバーのツールは、エージェントからは `mcp_<server>_<tool>` という名前で見えます。例: `mcp_Local_search`。
- **シークレット。** `${VAR}` (または `${VAR:-default}`) は環境変数から埋め込まれるので、キーをファイルに書く必要はありません。
- **許可。** MCP ツールも、他のツールと同じように許可を求めます。ただし、サーバーがそのツールを読み取り専用としてマークしている場合は除きます。
- **サーバーをオフにする。** 1 つのサーバーをスキップするには `"disabled": true` を追加し、すべてをスキップするには `--no-mcp` を付けて実行します。
- **安全性。** 暗号化されていない `http://` は localhost でのみ許可されます。リモートサーバーには `https://` が必要です。

`/mcp` と入力すると、接続されたサーバー、そのツール、エラーがあればその内容を確認できます。

---

## 保存場所

すべて `~/.agentiloop/` に保存されます。別のフォルダーを使いたい場合 (例えばテスト用の別プロファイル) は、`AGENTILOOP_HOME` を設定してください。

| ファイル | 内容 |
|---|---|
| `settings.json` | 記憶されたプロバイダー、モデル、オプションに加え、ウィザードが他の場所に書いた内容 |
| `env` | ウィザードが書いたあなたのキー (`KEY=value`、ファイルモード 600)。起動時に読み込まれ、シェルの `export` が優先されます |
| `sessions/` | 会話。1 つの会話につき 1 ファイル |
| `mcp.json` | MCP サーバー |
| `history.txt` | 入力したプロンプト (↑ / ↓ 用) |

---

## その他の環境変数

これらが必要になることはめったにありません:

| 変数 | 用途 |
|---|---|
| `ANTHROPIC_BASE_URL` | Anthropic へのリクエストをプロキシや互換サーバーに送ります |
| `ANTHROPIC_OAUTH_TOKEN` | Claude Code のトークン用に、`ANTHROPIC_API_KEY` の代わりに使えます |
| `OMLX_BASE_URL`、`OMLX_PORT`、`OMLX_API_KEY` | oMLX サーバーのアドレスとキー。デフォルトで読み込まれる `~/.omlx/settings.json` より優先されます (どちらも設定されていない場合はポート 8000) |
| `RUST_LOG=debug` | リクエストごとのトークン使用量を含むデバッグログを表示します |

---

## 開発者向け

### ビルドとテスト

```sh
cargo build --release     # → target/release/agentiloop
cargo test --workspace    # オフラインで実行、API キー不要
```

テストはネットワークにアクセスしません。エージェントループはスクリプト化された偽のモデルに対して、ストリーミングパーサーはローカルのテストサーバーに対して実行されます。MCP クライアントは、同梱のサンプルサーバーを使って 3 種類すべての接続方式 (stdio、HTTP、SSE) でテストされます。このサーバーを自分で起動して、手動で MCP を試すこともできます:

```sh
cargo run -p agentiloop-mcp --example mcp-example-server -- --http 8791   # または --sse 8792、または --stdio
```

### コードの構成

プロジェクトは 5 つのクレートに分かれていて、それぞれが前のクレートの上に構築されています:

| クレート | 内容 |
|---|---|
| `agentiloop-core` | 中核部分: エージェントループ、メッセージ、ツールとプロバイダーのインターフェース、許可、セッション、要約 |
| `agentiloop-provider` | モデルとの通信: Anthropic、OpenAI 互換サーバー、oMLX |
| `agentiloop-tools` | 組み込みツール: `read_file`、`write_file`、`edit_file`、`list_dir`、`bash` |
| `agentiloop-mcp` | MCP クライアント。Agent! の Swift 製 AgentMCP から移植したものです |
| `agentiloop-cli` | `agentiloop` プログラム: オプション、チャット、TUI、設定 |

### 依存関係

依存関係は少なく抑えています: 外部クレートは 18 個で、各クレートは実際に使うものだけを列挙しています。

| クレート | 用途 |
|---|---|
| `tokio`、`futures`、`async-trait` | 並行処理 |
| `serde`、`serde_json` | JSON の読み書き |
| `reqwest` | モデルや MCP サーバーへの HTTP リクエスト |
| `anyhow`、`thiserror` | エラー処理 |
| `tracing`、`tracing-subscriber` | ログ出力 |
| `dirs` | ホームフォルダーの特定 |
| `clap` | コマンドラインオプション |
| `rustyline` | 1 行ずつのチャット |
| `ratatui`、`unicode-width`、`textwrap` | TUI |
| `pulldown-cmark`、`syntect` | Markdown とコードのハイライト |

---

## ロードマップ

- [x] ストリーミング応答
- [x] OpenAI 互換プロバイダー
- [x] 長い会話の要約
- [x] 会話の保存
- [x] フルスクリーン TUI
- [x] MCP クライアント
- [ ] 次は何でしょう？

## ライセンス

[PolyForm Noncommercial 1.0.0](LICENSE)。このソフトウェアは、個人的かつ非商用の目的であれば、使用、変更、共有することができます。商用バージョンの構築や販売を含む商用利用の権利は AgentiLoop に留保されています。商用ライセンスについては AgentiLoop にお問い合わせください。

---

<a href="https://fluxionai.world/register?source=github&campaign=aiagent&promo=AIAGENT"><img src="docs/sponsors/fluxion-ai-silver-ad_ja.svg" width="900" alt="Fluxion AI（シルバースポンサー）：GPT、Claude などの主要 AI モデルをひとつの統合 API で。公式 API 価格と比べて最大 70% お得、さらに $3 分の API クレジット。" /></a>
