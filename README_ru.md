<a href="https://fluxionai.world/register?source=github&campaign=aiagent&promo=AIAGENT"><img src="docs/sponsors/fluxion-ai-silver-ad_ru.svg" width="900" alt="Fluxion AI, серебряный спонсор: единый API для GPT, Claude и других ведущих моделей ИИ. Экономия до 70 % по сравнению с официальными ценами API и $3 кредитов на API." /></a>

# AgentiLoopCLI

🌐 [English](README.md) · [Español](README_es.md) · [Français](README_fr.md) · [Deutsch](README_de.md) · [中文 (简体)](README_zh.md) · [Русский](README_ru.md) · [한국어](README_ko.md) · [日本語](README_ja.md)

### 🎉 Мы выпустили релиз v0.0.2 для Mac, Windows и Linux!

---

**Попробуйте прямо сейчас!** Прочитайте README и посмотрите, сколько времени у вас уйдёт на то, чтобы запустить AgentiLoop. Если столкнётесь с проблемами, дайте нам знать. Мы будем очень рады вашим отзывам.

**Бонус:** есть и версия на Go: **AgentiLoopGo** → https://github.com/AgentiLoop/AgentiLoopGo

Попробуйте обе и расскажите нам, какая справляется лучше: **Rust или Go?** 🦀 vs 🐹

---

### 💖 Поддержите AgentiLoop

Нравится проект? Помогите AgentiLoop оставаться быстрым и кроссплатформенным. Поддержите нас на **[GitHub Sponsors → AgentiLoop](https://github.com/sponsors/AgentiLoop)**. Уровни поддержки и бонусы описаны в [руководстве для спонсоров](https://github.com/AgentiLoop/Agent/blob/main/docs/SPONSORSHIP.md).

[![Поддержать AgentiLoop](https://img.shields.io/badge/Sponsor-AgentiLoop-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white)](https://github.com/sponsors/AgentiLoop)

---

AgentiLoop — это ИИ-агент для программирования, который работает в вашем терминале, в духе Claude Code. Вы описываете, что хотите, обычным языком. Агент читает ваши файлы, редактирует код и выполняет команды, чтобы добиться результата, и спрашивает вашего разрешения, прежде чем что-либо изменить.

Он написан на Rust и работает на macOS, Linux и Windows. Он поддерживает Claude (Anthropic), OpenAI, локальные модели через Ollama или LM Studio, а также oMLX на Apple Silicon.

Создано с помощью AgentiLoop Agent! Это наше детище. Готовые сборки для macOS, Linux и Windows доступны на странице [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases), либо вы можете скомпилировать программу из исходников с помощью Rust.

<img src="docs/pong.png" width="900" alt="AgentiLoop пишет, собирает и запускает игру Pong в стиле Atari на SwiftUI по одному запросу: живой diff и окно запущенной игры" />

---

## ⚡ Первый запуск: мастер настройки сделает всё сам

Ничего не нужно настраивать вручную. При первом запуске `agentiloop` встроенный мастер настройки стартует сам. Он спросит, какого провайдера вы хотите (Claude, OpenAI, Ollama / LM Studio или oMLX), примет ваш API-ключ (ввод скрыт), проверит, что ключ работает, даст выбрать модель и сохранит всё в `~/.agentiloop`. Около минуты, никаких файлов конфигурации, никаких строк `export`.

<img src="docs/setup-wizard-tui.png" width="900" alt="Мастер настройки внутри полноэкранного интерфейса (TUI): провайдер, скрытый ключ API, проверка соединения, список моделей, где сохранить ключ, затем первый запрос" />

Запустить его снова можно в любой момент командой `agentiloop --setup` (добавьте `--tui` для полноэкранной версии или введите `/setup` внутри сеанса). `agentiloop --reset` забывает всё и начинает с чистого листа. Никогда раньше не устанавливали программу для командной строки? Следуйте разделу [Впервые здесь?](#-впервые-здесь-запуск-за-5-минут) ниже, шаг за шагом.

---

## 🚀 Впервые здесь? Запуск за 5 минут

Без Rust, без Go, без компиляции. Вы скачиваете один файл, указываете API-ключ и начинаете общаться. Выполняйте шаги по порядку.

### 1. Скачайте AgentiLoop

Сначала выясните, какой файл вам нужен:

| Ваш компьютер | Файл для загрузки |
|---|---|
| Mac с Apple Silicon (M1, M2, M3, M4…) | `agentiloop-macos-arm64.tar.gz` |
| Mac с процессором Intel | `agentiloop-macos-x86_64.tar.gz` |
| Linux, 64-битный ПК | `agentiloop-linux-x86_64.tar.gz` |
| Linux на ARM (Raspberry Pi 4/5, ARM-серверы) | `agentiloop-linux-arm64.tar.gz` |
| Windows 10/11 | `agentiloop-windows-x86_64.zip` |

Не уверены? На Mac или Linux выполните `uname -m`. `arm64` или `aarch64` означает **arm64**, а `x86_64` — **x86_64**.

**macOS и Linux.** Откройте Терминал и вставьте эти строки. В примере используется файл для Apple Silicon, поэтому, если у вас другой, замените `macos-arm64` в первых трёх строках:

```sh
curl -LO https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.2/agentiloop-macos-arm64.tar.gz
tar xzf agentiloop-macos-arm64.tar.gz
mkdir -p ~/.local/bin && mv agentiloop-macos-arm64/agentiloop ~/.local/bin/
```

Так программа окажется в `~/.local/bin` — папке в вашем домашнем каталоге. Мастер настройки на шаге 3 предложит указать терминалу искать её там.

**Windows.** Откройте **PowerShell** (меню «Пуск» → введите "PowerShell") и вставьте:

```powershell
Invoke-WebRequest https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.2/agentiloop-windows-x86_64.zip -OutFile agentiloop.zip
Expand-Archive agentiloop.zip -DestinationPath $HOME\agentiloop -Force
$p = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$p;$HOME\agentiloop\agentiloop-windows-x86_64", "User")
```

Последние две строки добавляют AgentiLoop в ваш PATH. **Закройте PowerShell и откройте новое окно**, чтобы изменение вступило в силу.

### 2. Получите API-ключ

AgentiLoop — это агент. «Мозг» — это ИИ-модель, к которой вы его подключаете. Выберите **одну**:

| Вариант | Где получить | Стоимость |
|---|---|---|
| **Claude** (рекомендуется) | [console.anthropic.com](https://console.anthropic.com/settings/keys) → *Create Key*. Ключ начинается с `sk-ant-` | Оплата по факту использования |
| **OpenAI** | [platform.openai.com/api-keys](https://platform.openai.com/api-keys). Ключ начинается с `sk-` | Оплата по факту использования |
| **Ollama** (работает на вашем собственном компьютере) | Установите с [ollama.com](https://ollama.com), затем выполните `ollama pull qwen2.5-coder` | Бесплатно, ключ не нужен |

Сохраните ключ в надёжном месте. Он понадобится вам на следующем шаге.

### 3. Запустите мастер настройки

Вам не нужно редактировать файлы или вводить команды `export`. В AgentiLoop встроен мастер настройки, который задаст несколько вопросов и сохранит ключ за вас.

Поскольку `~/.local/bin` ещё нет в вашем PATH, в этот единственный раз запустите программу по полному пути (в Windows шаг 1 уже настроил PATH, так что просто введите `agentiloop`):

```sh
~/.local/bin/agentiloop
```

В первый раз, когда ключ ещё не настроен, мастер запускается сам. Это занимает около минуты, и он спрашивает пять вещей:

1. **Какого провайдера** — Claude, OpenAI, локальный OpenAI-совместимый сервер (Ollama, LM Studio, …) или oMLX. Введите номер.
2. **Ваш API-ключ** — ввод скрыт, на экране ничего не отображается. Локальным серверам ключ обычно не нужен; для oMLX на том же Mac мастер берёт ключ из собственных настроек oMLX, так что даже не спрашивает.
3. **Проверка соединения** — мастер сразу обращается к провайдеру. Если ключ неверный, он сообщит об этом и предложит попробовать ещё раз; ничего не сохраняется, пока соединение не заработает.
4. **Какую модель** — выберите одну из списка, который вернул провайдер, или нажмите Enter для модели по умолчанию. Позже её можно в любой момент сменить командой `/model`.
5. **Где хранить ключ** — нажмите Enter для варианта по умолчанию, `~/.agentiloop/env`, приватного файла, который читает только AgentiLoop. (Остальные варианты — для тех, кому ключ нужен ещё и в оболочке или в Связке ключей macOS — описаны в разделе [Продвинутый вариант: задать ключ вручную](#шаг-2-подключите-модель) в «Быстром старте».)

В конце мастер заметит, что `~/.local/bin` нет в вашем PATH, и предложит добавить его. Нажмите **Enter** (да). Затем он выводит `All set` и переводит вас к строке ввода. Вот полный прогон в обычном терминале с выбором Claude и значениями по умолчанию (ваш список моделей будет отличаться):

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

Можно начать общение прямо здесь или ввести `/exit` и перейти к шагу 4. С этого момента простой `agentiloop` запускается сразу со строки ввода.

Для Claude можно вставить либо обычный API-ключ (`sk-ant-api…`), либо токен Claude Code (`sk-ant-oat01-…`, который выдаёт `claude setup-token`); AgentiLoop сам определит, какой из них вы используете. Если вы выбрали вариант 3 (Ollama, LM Studio, …), мастер также спросит URL сервера и предложит `http://localhost:11434/v1` по умолчанию, так что для локальной Ollama достаточно нажать Enter.

**Тот же мастер, на весь экран.** Мастер работает в том интерфейсе, которым вы пользуетесь. Запустите его с `--tui`, и те же вопросы появятся внутри полноэкранного интерфейса; по завершении вы уже находитесь у строки ввода:

<img src="docs/setup-wizard-tui.png" width="900" alt="Мастер настройки внутри полноэкранного интерфейса (TUI): провайдер, скрытый ключ API, проверка соединения, список моделей, где сохранить ключ, затем первый запрос" />

**Запустить заново или переделать можно в любой момент:**

```bash
agentiloop --setup          # мастер в обычном терминале
agentiloop --setup --tui    # мастер внутри полноэкранного TUI (как на скриншоте)
/setup                      # из открытой сессии (REPL или TUI)
agentiloop --reset          # забыть всё и начать с чистого листа
```

> 🔒 **Храните ключ в секрете.** `~/.agentiloop/env` может прочитать только вы. Не вставляйте ключ в чаты и не добавляйте его в git. На Mac вариант 3 в вопросе «Where should the credential be saved?» сохраняет ключ в Связке ключей, так что он никогда не лежит на диске в открытом виде.

Предпочитаете управлять ключом сами через переменные среды? Это тоже возможно, но это продвинутый путь; см. [Продвинутый вариант: задать ключ вручную](#шаг-2-подключите-модель) в разделе «Быстрый старт».


### 4. Проверьте, что всё работает

**Откройте новое окно терминала**, чтобы оно подхватило изменение PATH, сделанное мастером (Windows: новое окно PowerShell). Затем:

```sh
agentiloop --version
```

Вы должны увидеть `agentiloop 0.0.2`. Теперь запустите программу без параметров:

```sh
agentiloop
```

Она должна сразу перейти к строке ввода. Если вместо этого снова запускается мастер, ключ не сохранился: пройдите шаг 3 ещё раз.

### 5. Ваш первый сеанс

Перейдите в папку проекта и запустите полноэкранный интерфейс:

Начните с новой пустой тестовой папки, чтобы спокойно всё попробовать. Для настоящего проекта вместо этого перейдите в его папку с помощью `cd` (например, `cd ~/code/my-app`).

```sh
mkdir -p ~/agentiloop-test
cd ~/agentiloop-test
agentiloop --tui
```

Используете **Ollama**? Мастер уже запомнил сервер и выбранную вами модель. Чтобы переключиться на другую скачанную модель, введите `/model` внутри сеанса.

Теперь просто опишите обычными словами, что вы хотите, и нажмите **Enter**. Несколько хороших запросов для начала:

```text
объясни, что делает этот проект
перечисли файлы в src и скажи, какой из них точка входа
найди комментарии TODO и кратко опиши их
добавь флаг --verbose в парсер командной строки
запусти тесты и исправь всё, что не проходит
создай README.md для этого проекта
```

Прежде чем изменить файл или выполнить команду, агент спрашивает вас. Нажмите **y** — «да», **n** — «нет», **a** — всегда разрешать этот инструмент в текущем сеансе, или **Esc**, чтобы пропустить шаг. Нажмите **Ctrl-C**, чтобы выйти. В следующий раз простая команда `agentiloop` запустится так же и продолжит ваш последний разговор.

Нужен всего один ответ без чата? Передайте вопрос в качестве аргумента:

```sh
agentiloop "explain what this project does"
```

### Что он умеет? (инструменты)

Агент работает с пятью встроенными инструментами. Вам не нужно вызывать их самостоятельно. Вы описываете цель, а агент сам выбирает инструмент:

| Инструмент | Что делает | Спрашивает заранее? |
|---|---|---|
| `read_file` | Читает файл (с номерами строк) | Нет |
| `list_dir` | Выводит список файлов в папке | Нет |
| `write_file` | Создаёт новый файл или перезаписывает существующий | **Да** |
| `edit_file` | Изменяет точный фрагмент текста в файле | **Да** |
| `bash` | Выполняет команду оболочки, например тесты, сборку или `git` (`sh -c` на Mac/Linux, `cmd /C` в Windows) | **Да** |

Нужно больше инструментов, например веб-поиск, базы данных или GitHub? Добавьте MCP-серверы; см. [Добавление инструментов через MCP](#добавление-инструментов-через-mcp-необязательно).

### Команда справки

`agentiloop --help` выводит все параметры:

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

Внутри сеанса введите `/help`, чтобы увидеть команды чата (`/model`, `/sessions`, `/resume`, `/clear`, `/compact`, `/mcp`, `/exit`). Полный справочник — в разделах [Все параметры](#все-параметры) и [Команды в чате](#команды-в-чате).

### Что-то не получается? Быстрые решения

| Вы видите | Решение |
|---|---|
| `command not found: agentiloop` | `~/.local/bin` нет в вашем PATH. Сначала откройте новое окно терминала; если это не помогло, выполните `~/.local/bin/agentiloop --setup` и ответьте «да», когда мастер предложит добавить папку в PATH. В Windows откройте новое окно PowerShell |
| `Error: no provider credentials found` | Ключ не сохранён. Выполните `agentiloop --setup` (шаг 3), затем проверьте результат по шагу 4 |
| macOS: *"agentiloop" cannot be opened* / *unidentified developer* | Так бывает, если вы скачали файл через браузер, а не через `curl`. Выполните `xattr -d com.apple.quarantine ~/.local/bin/agentiloop` |
| Windows: *Windows protected your PC* | Нажмите **More info** → **Run anyway** |
| `401` / `invalid x-api-key` / ошибка аутентификации | Ключ неверный или был вставлен с пробелами либо кавычками. Скопируйте его ещё раз и выполните `agentiloop --setup`, чтобы ввести заново |
| Ollama: model not found | Выполните `ollama list` и передайте точное имя через `-m` |
| Программа продолжает использовать старую модель или провайдера | Она запоминает ваш последний выбор. Передайте `-p` / `-m`, чтобы изменить его, или выполните `agentiloop --reset`, чтобы начать с чистого листа |

Всё ещё не получается? [Создайте issue](https://github.com/AgentiLoop/AgentiLoopCLI/issues) и вставьте команду и текст ошибки. Мы поможем.

---

## Быстрый старт

Три шага: установите, подключите модель, запустите.

### Шаг 1: Установите

**Загрузка:** скачайте архив для вашей платформы со страницы [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases), распакуйте его и поместите `agentiloop` (`agentiloop.exe` в Windows) в PATH.

**Или соберите сами:** если у вас ещё нет Rust, установите его с [rustup.rs](https://rustup.rs). Затем:

```sh
git clone https://github.com/AgentiLoop/AgentiLoopCLI.git
cd AgentiLoopCLI
cargo install --path crates/agentiloop-cli
```

Эта команда соберёт программу и добавит команду `agentiloop` в ваш PATH, в `~/.cargo/bin`.

> **Не хотите устанавливать?** Всё в этом README работает и прямо из папки репозитория. Везде, где вы видите `agentiloop <options>`, вводите вместо этого `cargo run -- <options>`. Всё, что идёт после `--`, передаётся AgentiLoop.

### Шаг 2: Подключите модель

AgentiLoop нужна модель, с которой можно общаться. **Для этого не нужно задавать никаких переменных среды**: просто запустите `agentiloop`, и встроенный мастер настройки задаст несколько вопросов и сохранит всё за вас. Пошаговое руководство с полным прогоном — выше, в разделе [3. Запустите мастер настройки](#3-запустите-мастер-настройки).

<details>
<summary><b>Продвинутый вариант: задать ключ вручную</b> (пропустите, если мастер у вас сработал)</summary>

Если вы предпочитаете управлять ключом сами или запускаете AgentiLoop в скрипте или CI, где некому отвечать мастеру, задайте одну из этих переменных среды, и AgentiLoop использует её без вопросов:

| Я хочу использовать… | Что задать |
|---|---|
| **Claude** (Anthropic) | `export ANTHROPIC_API_KEY=sk-ant-...` |
| **OpenAI** | `export OPENAI_API_KEY=sk-...` |
| **Ollama, LM Studio** или любой OpenAI-совместимый сервер | `export OPENAI_BASE_URL=http://localhost:11434/v1` (адрес вашего сервера; для локальных серверов ключ не нужен) |
| **oMLX** (локальные модели на Apple Silicon) | Обычно ничего. Запустите oMLX, затем запустите AgentiLoop с `-p omlx` |

**Подробнее про oMLX.** Когда oMLX работает на том же Mac, AgentiLoop считывает порт сервера и API-ключ из собственного файла настроек oMLX (`~/.omlx/settings.json`). Если oMLX работает на другой машине или вы хотите переопределить эти настройки:

```sh
export OMLX_BASE_URL=http://192.168.1.50:7777/v1   # адрес сервера oMLX (или OMLX_PORT=7777 для localhost)
export OMLX_API_KEY=...                            # API-ключ из настроек oMLX
```

Если в oMLX отключена проверка API-ключа, ключ не нужен.

`export` действует только в той вкладке терминала, где вы его ввели. Чтобы настройка сохранилась навсегда, эту строку нужно добавить в профиль оболочки (`~/.zshrc` на macOS) — именно это и делает за вас вариант мастера **«Also add it to ~/.zshrc»**. Аналогично, вариант мастера **«macOS Keychain»** — это автоматическая версия вот этого:

```sh
# один раз: сохраните ключ в Связке ключей
security add-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w "sk-ant-..."

# в ~/.zshrc: загружать его в каждом новом терминале
export ANTHROPIC_API_KEY="$(security find-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w 2>/dev/null)"
```

</details>

### Шаг 3: Запустите

Перейдите в проект, над которым хотите работать, и запустите AgentiLoop:

```sh
cd ~/my-project
agentiloop --tui
```

`--tui` открывает полноэкранный интерфейс — мы рекомендуем именно его. Введите, что вы хотите, например *«найди, где загружается файл конфигурации, и добавь флаг --verbose»*, и нажмите Enter.

Вы увидите ответы агента, каждый инструмент, который он использует (🔧), и каждый результат (✓ или ✖). Поле внизу показывает, чем агент занят прямо сейчас, например ` ✻ Thinking...  12s `. Прежде чем записать файл или выполнить команду, он спрашивает вас:

- **y**: да, на этот раз
- **n**: нет
- **a**: всегда разрешать этот инструмент до конца сеанса
- **Esc**: пропустить этот шаг, но продолжить работу

---

## Три способа использования

| Режим | Команда | Для чего подходит |
|---|---|---|
| **TUI** (полный экран) | `agentiloop --tui` | Повседневная работа: прокручиваемая история, статус в реальном времени, кликабельные ссылки |
| **Чат** (построчно) | `agentiloop` | Простые терминалы или если вы предпочитаете обычный текст |
| **Разовый запрос** | `agentiloop "explain this project"` | Один вопрос: программа отвечает и завершает работу. Удобно в скриптах |

Клавиши в TUI: **Enter** — отправить · **↑ / ↓** — листать предыдущие запросы · **PgUp / PgDn** или колесо мыши — прокрутка · **Ctrl-U** — очистить строку · **Ctrl-C** — выход.

---

## Программа запоминает ваши настройки

Параметры нужно ввести всего один раз. AgentiLoop сохраняет то, как вы его запустили, поэтому в следующий раз простая команда `agentiloop` запустится точно так же:

```sh
agentiloop -p anthropic --tui    # первый раз: выберите провайдера и TUI
agentiloop                       # дальше: тот же провайдер, та же модель, TUI и ваш последний разговор
```

Что запоминается:

- **Провайдер** (`-p`) и **TUI вкл./выкл.** (`--tui` / `--no-tui`)
- **Модель**: последняя использованная, отдельно для каждого провайдера. При возврате к провайдеру возвращается и его модель.
- **Лимиты**: `--max-turns` и `--compact-at`
- **Ваш разговор**: программа продолжает последний разговор в текущей папке, если в нём использовался тот же провайдер. Прежние сообщения снова отображаются на экране, так что вы можете прокрутить назад и увидеть, на чём остановились

Чтобы что-то изменить, передайте новый параметр. Он применяется сразу и запоминается на будущее:

```sh
agentiloop -p omlx        # переключиться на oMLX (его последняя модель тоже вернётся)
agentiloop -m <model>     # сменить модель
agentiloop --no-tui       # вернуться к построчному чату
agentiloop --new          # начать новый разговор (старый останется сохранённым)
```

Некоторые вещи намеренно **никогда** не запоминаются:

- `--yes`: пропуск запросов разрешения каждый раз должен быть осознанным выбором
- `--no-mcp`, `-C` и разовые запросы
- API-ключи: они хранятся в `~/.agentiloop/env` (его записывает мастер) или в окружении вашей оболочки, но никогда в `settings.json`

Чтобы сбросить всё, выполните `agentiloop --reset`.

---

## Все параметры

Каждый параметр можно также задать через переменную среды, указанную во втором столбце. Параметр, введённый вручную, всегда важнее запомненного значения.

| Параметр | Переменная среды | Что делает |
|---|---|---|
| `-p, --provider <name>` | `AGENTILOOP_PROVIDER` | `anthropic`, `openai` или `omlx`. Если вы его не укажете, AgentiLoop использует последний или определит его по вашим ключам (сначала Anthropic, затем OpenAI, затем oMLX) |
| `-m, --model <id>` | `AGENTILOOP_MODEL` | Какую модель использовать |
| `--tui` / `--no-tui` | `AGENTILOOP_TUI` | Включить / выключить полноэкранный интерфейс |
| `--new` | | Начать новый разговор вместо продолжения |
| `-c, --continue` | | Продолжить последний разговор здесь (уже по умолчанию) |
| `-r, --resume <id>` | | Снова открыть конкретный разговор (id можно найти с помощью `/sessions`) |
| `-C, --cwd <folder>` | | Работать в другой папке, а не в той, где вы находитесь |
| `--yes` | `AGENTILOOP_YES` | Не спрашивать перед запуском инструментов. ⚠️ Только для доверенного автоматического использования |
| `--no-mcp` | `AGENTILOOP_NO_MCP` | Не запускать MCP-серверы (см. ниже) |
| `--setup` | | Снова запустить мастер первоначальной настройки (провайдер, ключ, модель). Вместе с `--tui` он запускается внутри полноэкранного интерфейса |
| `--reset` | | Как с нуля: удаляет `~/.agentiloop`, блок agentiloop в профиле оболочки и записи Связки ключей, созданные мастером (в Windows — заданные им пользовательские переменные окружения). Написанные вручную строки `export` только комментируются, и только с вашего согласия. Добавьте `--yes`, чтобы пропустить вопросы |
| `--max-turns <n>` | | Максимальное число шагов агента на один запрос (по умолчанию 50) |
| `--compact-at <tokens>` | `AGENTILOOP_COMPACT_AT` | Когда сжимать длинный разговор в краткое изложение (по умолчанию 150000, `0` = никогда) |
| `-h` / `-V` | | Справка / версия |

Несколько примеров:

```sh
agentiloop -p openai -m gpt-4o-mini "summarize this repo"    # один вопрос с конкретной моделью
agentiloop -C ../other-repo --tui                            # работа над другим проектом
agentiloop --yes "run the tests and fix any failures"        # без присмотра, без запросов
```

**Какая модель используется?** Побеждает первый подходящий вариант:

1. `-m` в командной строке
2. модель разговора, который вы продолжаете
3. последняя модель, которую вы использовали с этим провайдером
4. модель провайдера по умолчанию: `claude-sonnet-5` для Anthropic, `gpt-4o-mini` для OpenAI или первая модель, которую предлагает oMLX

---

## Команды в чате

Вводите их в строке запроса, в TUI или в чате:

| Команда | Что делает |
|---|---|
| `/model` | Показывает доступные модели. `/model 3` или `/model <id>` переключает модель (и запоминает выбор) |
| `/sessions` | Выводит список сохранённых разговоров, начиная с самых новых |
| `/resume <n or id>` | Снова открывает один из них |
| `/clear` | Очищает разговор и начинает новый |
| `/compact` | Сразу сжимает разговор в краткое изложение, чтобы освободить место |
| `/mcp` | Показывает подключённые MCP-серверы и их инструменты |
| `/help` | Выводит список этих команд |
| `/exit` | Выход |

---

## Длинные разговоры

Модели могут прочитать за раз лишь ограниченный объём текста. Когда разговор становится большим (по умолчанию — когда запрос достигает 150 000 токенов), AgentiLoop просит модель кратко изложить всё сказанное и продолжает работу на основе этого изложения. Когда это происходит, вы увидите пометку 📦. `/compact` делает это по запросу, а `--compact-at 0` отключает эту функцию.

---

## Добавление инструментов через MCP (необязательно)

Серверы [MCP](https://modelcontextprotocol.io) дают агенту дополнительные инструменты, например доступ к базам данных, веб-поиск или ваши собственные скрипты. Перечислите их в JSON-файле:

- `~/.agentiloop/mcp.json`: доступны во всех проектах
- `.mcp.json` в папке проекта: только в этом проекте. Если одно и то же имя есть в обоих файлах, приоритет у этого файла.

Формат тот же, что используют Claude Code, Claude Desktop и Agent!, так что вы можете скопировать существующие конфигурации:

```json
{ "mcpServers": {
    "Local":  { "command": "my-mcp-server", "args": ["--flag"], "env": { "API_KEY": "${MY_KEY}" } },
    "Remote": { "url": "https://example.com/mcp", "headers": { "Authorization": "Bearer ${TOKEN}" } }
} }
```

Как это работает:

- **Два вида серверов.** Сервер с `command` — это локальная программа, которую AgentiLoop запускает за вас. К серверу с `url` программа обращается по HTTP. Работают и новые серверы "Streamable HTTP", и старые "SSE"; URL, оканчивающийся на `/sse` (или `"transport": "sse"`), выбирает старый вариант.
- **Имена инструментов.** Каждый инструмент сервера виден агенту как `mcp_<server>_<tool>`, например `mcp_Local_search`.
- **Секреты.** `${VAR}` (или `${VAR:-default}`) подставляется из ваших переменных среды, так что ключи не нужно хранить в файле.
- **Разрешения.** MCP-инструменты запрашивают разрешение, как и любые другие, если только сервер не пометил инструмент как доступный только для чтения.
- **Отключение серверов.** Добавьте `"disabled": true`, чтобы пропустить один сервер, или запустите программу с `--no-mcp`, чтобы пропустить их все.
- **Безопасность.** Обычный `http://` разрешён только для localhost; удалённым серверам нужен `https://`.

Введите `/mcp`, чтобы увидеть, какие серверы подключены, их инструменты и возможные ошибки.

---

## Где всё хранится

Всё хранится в `~/.agentiloop/`. Задайте `AGENTILOOP_HOME`, чтобы использовать другую папку, например отдельный тестовый профиль.

| Файл | Что в нём |
|---|---|
| `settings.json` | Запомненные провайдер, модели и параметры, а также то, что мастер записал в другие места |
| `env` | Ваш ключ, записанный мастером (`KEY=value`, режим файла 600). Загружается при запуске; `export` в оболочке имеет приоритет |
| `sessions/` | Ваши разговоры, по одному файлу на каждый |
| `mcp.json` | Ваши MCP-серверы |
| `history.txt` | Запросы, которые вы вводили (для ↑ / ↓) |

---

## Другие переменные среды

Они вам понадобятся редко:

| Переменная | Назначение |
|---|---|
| `ANTHROPIC_BASE_URL` | Отправлять запросы Anthropic через прокси или совместимый сервер |
| `ANTHROPIC_OAUTH_TOKEN` | Альтернатива `ANTHROPIC_API_KEY` для токена Claude Code |
| `OMLX_BASE_URL`, `OMLX_PORT`, `OMLX_API_KEY` | Адрес и ключ сервера oMLX. Они переопределяют `~/.omlx/settings.json`, который читается по умолчанию (порт 8000, если ни то, ни другое не задано) |
| `RUST_LOG=debug` | Показывать отладочные логи, включая расход токенов на каждый запрос |

---

## Для разработчиков

### Сборка и тесты

```sh
cargo build --release     # → target/release/agentiloop
cargo test --workspace    # работает офлайн, API-ключи не нужны
```

Тесты не обращаются к сети. Цикл агента работает с заранее запрограммированной фиктивной моделью, а потоковые парсеры — с локальным тестовым сервером. MCP-клиент тестируется со встроенным примером сервера по всем трём типам подключения (stdio, HTTP, SSE). Вы можете сами запустить этот сервер, чтобы попробовать MCP вручную:

```sh
cargo run -p agentiloop-mcp --example mcp-example-server -- --http 8791   # или --sse 8792, или --stdio
```

### Как устроен код

Проект разделён на пять крейтов, и каждый из них опирается на предыдущие:

| Крейт | Что в нём |
|---|---|
| `agentiloop-core` | Сердце проекта: цикл агента, сообщения, интерфейсы инструментов и провайдеров, разрешения, сеансы, сжатие разговоров |
| `agentiloop-provider` | Общается с моделями: Anthropic, OpenAI-совместимые серверы, oMLX |
| `agentiloop-tools` | Встроенные инструменты: `read_file`, `write_file`, `edit_file`, `list_dir`, `bash` |
| `agentiloop-mcp` | MCP-клиент, перенесённый из AgentMCP на Swift из Agent! |
| `agentiloop-cli` | Программа `agentiloop`: параметры, чат, TUI, настройки |

### Зависимости

Мы стараемся свести зависимости к минимуму: 18 внешних крейтов, и каждый крейт подключает только те, что ему действительно нужны.

| Крейты | Для чего |
|---|---|
| `tokio`, `futures`, `async-trait` | Параллельное выполнение задач |
| `serde`, `serde_json` | Чтение и запись JSON |
| `reqwest` | HTTP-запросы к моделям и MCP-серверам |
| `anyhow`, `thiserror` | Обработка ошибок |
| `tracing`, `tracing-subscriber` | Логирование |
| `dirs` | Поиск вашей домашней папки |
| `clap` | Параметры командной строки |
| `rustyline` | Построчный чат |
| `ratatui`, `unicode-width`, `textwrap` | TUI |
| `pulldown-cmark`, `syntect` | Markdown и подсветка кода |

---

## Дорожная карта

- [x] Потоковые ответы
- [x] OpenAI-совместимые провайдеры
- [x] Сжатие длинных разговоров
- [x] Сохранённые разговоры
- [x] Полноэкранный TUI
- [x] MCP-клиент
- [ ] Что дальше?

## Лицензия

[PolyForm Noncommercial 1.0.0](LICENSE). Вы можете использовать, изменять и распространять это программное обеспечение в личных и некоммерческих целях. Коммерческое использование, включая создание или продажу коммерческих версий, остаётся за AgentiLoop. Чтобы получить коммерческую лицензию, свяжитесь с AgentiLoop.

---

<a href="https://fluxionai.world/register?source=github&campaign=aiagent&promo=AIAGENT"><img src="docs/sponsors/fluxion-ai-silver-ad_ru.svg" width="900" alt="Fluxion AI, серебряный спонсор: единый API для GPT, Claude и других ведущих моделей ИИ. Экономия до 70 % по сравнению с официальными ценами API и $3 кредитов на API." /></a>
