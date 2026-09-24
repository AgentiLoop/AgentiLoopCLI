# AgentiLoopCLI

🌐 [English](README.md) · [Español](README_es.md) · [Français](README_fr.md) · [Deutsch](README_de.md) · [中文 (简体)](README_zh.md) · [Русский](README_ru.md) · [한국어](README_ko.md) · [日本語](README_ja.md)

### Mac、Windows、Linux 向けのプレリリース版 v0.0.1 を公開しました！

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
curl -LO https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.1/agentiloop-macos-arm64.tar.gz
tar xzf agentiloop-macos-arm64.tar.gz
mkdir -p ~/.local/bin && mv agentiloop-macos-arm64/agentiloop ~/.local/bin/
```

これでプログラムがホームディレクトリ内のフォルダー `~/.local/bin` に置かれます。ステップ 3 で、ターミナルがそこを探すように設定します。

**Windows。** **PowerShell** を開き (スタートメニュー → 「PowerShell」と入力)、次を貼り付けます:

```powershell
Invoke-WebRequest https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.1/agentiloop-windows-x86_64.zip -OutFile agentiloop.zip
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

### 3. 設定をシェルプロファイルに保存する

**シェルプロファイル**とは、ターミナルが新しいウィンドウを開くたびに読み込む小さなテキストファイルです。ここに設定を書いておけば、この作業は一度だけで済みます。これを省くと、新しいターミナルを開くたびにキーを入力し直さなければなりません。これが、つまずく原因の第 1 位です。

**どのファイルですか？**

| システム | シェル (デフォルト) | プロファイルファイル |
|---|---|---|
| macOS (Catalina 10.15 以降) | zsh | `~/.zshrc` |
| ほとんどの Linux ディストリビューション | bash | `~/.bashrc` |
| zsh を使っている Linux または Mac | zsh | `~/.zshrc` |
| fish シェル | fish | `~/.config/fish/config.fish` |
| Windows | PowerShell | 不要、下記参照 |

どのシェルを使っているかわからない場合は、`echo $SHELL` を実行してください。`~` はホームフォルダーを意味するので、`~/.zshrc` は例えば `/Users/you/.zshrc` です。ドットで始まるファイルは Finder やファイルブラウザーでは非表示になりますが、これは正常です。

**ファイルを開きます。** 次のどちらかを使ってください (ファイルがまだなければ作成されます):

```sh
nano ~/.zshrc                        # どこでも使えます、ターミナル内で直接編集
touch ~/.zshrc && open -e ~/.zshrc   # macOS: テキストエディットで開きます
```

(bash を使っている Linux の場合: `~/.zshrc` の代わりに `~/.bashrc` を使ってください。)

**次の行を末尾に追加します。** 必要なキーの行だけを残し、引用符の間に実際のキーを貼り付けてください:

```sh
# AgentiLoop
export PATH="$HOME/.local/bin:$PATH"

export ANTHROPIC_API_KEY="sk-ant-paste-your-key-here"          # Claude
# export OPENAI_API_KEY="sk-paste-your-key-here"               # OpenAI
# export OPENAI_BASE_URL="http://localhost:11434/v1"           # Ollama (キー不要)
```

**保存して閉じます。** nano の場合: **Ctrl-O**、**Enter**、そして **Ctrl-X**。テキストエディットの場合: **⌘S** を押してからウィンドウを閉じます。

**読み込みます。** 新しいターミナルウィンドウを開くか、次を実行してください:

```sh
source ~/.zshrc
```

**fish** は構文が異なります。次を `~/.config/fish/config.fish` に書いてください:

```fish
fish_add_path $HOME/.local/bin
set -gx ANTHROPIC_API_KEY "sk-ant-paste-your-key-here"
```

**Windows (PowerShell)。** Windows には、このために編集するプロファイルファイルはありません。代わりに、キーをユーザー環境変数として保存します:

```powershell
setx ANTHROPIC_API_KEY "sk-ant-paste-your-key-here"
# または: setx OPENAI_API_KEY "sk-..."   /   setx OPENAI_BASE_URL "http://localhost:11434/v1"
```

その後、**PowerShell を閉じて新しいウィンドウを開いてください**。`setx` は、実行したウィンドウ自体には反映されません。マウス操作でも設定できます: スタート → *Edit environment variables for your account* → *New…*。

> 🔒 **キーは秘密にしてください。** プロファイルファイルを git にコミットしたり、キーをチャットに貼り付けたりしないでください。Mac では、代わりにキーチェーンに保存することもできます。[クイックスタートのステップ 2](#ステップ-2-モデルを接続する) を参照してください。

### 4. 動作を確認する

```sh
agentiloop --version
```

`agentiloop 0.0.1` と表示されるはずです。次に、キーが読み込まれているか確認します:

```sh
echo $ANTHROPIC_API_KEY | cut -c1-10    # macOS / Linux: sk-ant-... と表示されるはずです
```
```powershell
$env:ANTHROPIC_API_KEY.Substring(0,10)  # Windows PowerShell
```

何も表示されない場合は、ステップ 3 に戻ってください。キーがまだ読み込まれていません。

### 5. 最初のセッション

プロジェクトフォルダーに移動して、フルスクリーンのインターフェースを起動します:

まずは新しい空のテスト用フォルダーで、安心して試してみましょう。実際のプロジェクトで使うときは、代わりに `cd` でそのプロジェクトのフォルダーに移動してください（例：`cd ~/code/my-app`）。

```sh
mkdir -p ~/agentiloop-test
cd ~/agentiloop-test
agentiloop --tui
```

**Ollama** を使っていますか？ プロバイダーと、pull 済みのモデルを指定してください: `agentiloop -p openai -m qwen2.5-coder --tui`。

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
| `command not found: agentiloop` | `~/.local/bin` が PATH に入っていません。ステップ 3 の `export PATH=...` の行を追加してから、新しいターミナルを開いてください。Windows では、新しい PowerShell ウィンドウを開いてください |
| `Error: no provider credentials found` | キーが読み込まれていません。ステップ 3 をやり直し、ステップ 4 で確認してください |
| macOS: *"agentiloop" cannot be opened* / *unidentified developer* | `curl` ではなくブラウザーでダウンロードした場合に起こります。`xattr -d com.apple.quarantine ~/.local/bin/agentiloop` を実行してください |
| Windows: *Windows protected your PC* | **More info** → **Run anyway** をクリックしてください |
| `401` / `invalid x-api-key` / 認証エラー | キーが間違っているか、スペースや引用符が含まれています。もう一度コピーして、プロファイル内の行を確認してください |
| Ollama: model not found | `ollama list` を実行して、正確な名前を `-m` で渡してください |
| 古いモデルやプロバイダーが使われ続ける | 前回の選択を記憶しているためです。`-p` / `-m` を渡して変更するか、`~/.agentiloop/settings.json` を削除してリセットしてください |

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

AgentiLoop には、対話するためのモデルが必要です。次のいずれかを選んでください:

| 使いたいもの… | やること |
|---|---|
| **Claude** (Anthropic) | `export ANTHROPIC_API_KEY=sk-ant-...` |
| **OpenAI** | `export OPENAI_API_KEY=sk-...` |
| **Ollama、LM Studio**、または任意の OpenAI 互換サーバー | `export OPENAI_BASE_URL=http://localhost:11434/v1` (お使いのサーバーのアドレスを指定します。ローカルサーバーならキーは不要です) |
| **oMLX** (Apple Silicon 上のローカルモデル) | 通常は何もしなくて大丈夫です。oMLX を起動してから、`-p omlx` を付けて AgentiLoop を実行します (下記参照) |

Claude では、通常の API キー (`sk-ant-api…`) または Claude Code のトークン (`sk-ant-oat01-…`、`claude setup-token` で取得できます) を使えます。AgentiLoop がどちらの種類かを自動で判別します。

**oMLX の詳細。** oMLX が同じ Mac で動いている場合、AgentiLoop は oMLX 自身の設定ファイル (`~/.omlx/settings.json`) からサーバーのポートと API キーを読み込むので、何も export する必要はありません。oMLX が別のマシンで動いている場合や、その設定を上書きしたい場合は、自分で export してください:

```sh
export OMLX_BASE_URL=http://192.168.1.50:7777/v1   # oMLX サーバーのアドレス (localhost なら OMLX_PORT=7777)
export OMLX_API_KEY=...                            # oMLX の設定にある API キー
```

oMLX で API キーの検証がオフになっている場合、キーは不要です。

`export` は、入力したターミナルのタブでしか有効になりません。永続的にするには、その行をシェルプロファイル (macOS では `~/.zshrc`) に追加してください。Mac では、キーをファイルではなくキーチェーンに保存することもできます:

```sh
# 一度だけ: キーをキーチェーンに保存します
security add-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w "sk-ant-..."

# ~/.zshrc に記述: 新しいターミナルを開くたびに読み込みます
export ANTHROPIC_API_KEY="$(security find-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w 2>/dev/null)"
```

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
- API キー: これらはシェルプロファイルに保存したままにします

すべてを忘れさせるには、`~/.agentiloop/settings.json` を削除してください。

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
| `settings.json` | 記憶されたプロバイダー、モデル、オプション。削除するとリセットされます |
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
