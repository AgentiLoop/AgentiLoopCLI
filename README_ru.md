# AgentiLoopCLI

🌐 [English](README.md) · [Español](README_es.md) · [Français](README_fr.md) · [Deutsch](README_de.md) · [中文 (简体)](README_zh.md) · [Русский](README_ru.md) · [한국어](README_ko.md) · [日本語](README_ja.md)

### Мы выпустили предварительную сборку v0.0.1 для Mac, Windows и Linux!

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

<img width="2048" height="1152" alt="AgentiLoop пишет код" src="https://github.com/user-attachments/assets/d910bbd2-b47d-4c4a-89af-ac0753ccf279" />

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
curl -LO https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.1/agentiloop-macos-arm64.tar.gz
tar xzf agentiloop-macos-arm64.tar.gz
mkdir -p ~/.local/bin && mv agentiloop-macos-arm64/agentiloop ~/.local/bin/
```

Так программа окажется в `~/.local/bin` — папке в вашем домашнем каталоге. На шаге 3 вы укажете терминалу искать её там.

**Windows.** Откройте **PowerShell** (меню «Пуск» → введите "PowerShell") и вставьте:

```powershell
Invoke-WebRequest https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.1/agentiloop-windows-x86_64.zip -OutFile agentiloop.zip
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

### 3. Сохраните настройки в профиле оболочки

Ваш **профиль оболочки** — это небольшой текстовый файл, который терминал читает каждый раз при открытии нового окна. Запишите туда настройки, и делать это придётся всего один раз. Если пропустить этот шаг, вам придётся вводить ключ заново в каждом новом терминале. Это причина номер один, по которой у людей что-то не получается.

**Какой это файл?**

| Система | Оболочка (по умолчанию) | Файл профиля |
|---|---|---|
| macOS (Catalina 10.15 и новее) | zsh | `~/.zshrc` |
| Большинство дистрибутивов Linux | bash | `~/.bashrc` |
| Linux или Mac с zsh | zsh | `~/.zshrc` |
| Оболочка fish | fish | `~/.config/fish/config.fish` |
| Windows | PowerShell | не нужен, см. ниже |

Не знаете, какая у вас оболочка? Выполните `echo $SHELL`. `~` означает вашу домашнюю папку, так что `~/.zshrc` — это, например, `/Users/you/.zshrc`. Файлы, имя которых начинается с точки, скрыты в Finder и файловых менеджерах — это нормально.

**Откройте файл.** Воспользуйтесь одним из способов (если файла ещё нет, он будет создан):

```sh
nano ~/.zshrc                        # работает везде, прямо в терминале
touch ~/.zshrc && open -e ~/.zshrc   # macOS: открывает файл в TextEdit
```

(Linux с bash: используйте `~/.bashrc` вместо `~/.zshrc`.)

**Добавьте эти строки в конец файла.** Оставьте только нужную вам строку с ключом и вставьте свой настоящий ключ между кавычками:

```sh
# AgentiLoop
export PATH="$HOME/.local/bin:$PATH"

export ANTHROPIC_API_KEY="sk-ant-paste-your-key-here"          # Claude
# export OPENAI_API_KEY="sk-paste-your-key-here"               # OpenAI
# export OPENAI_BASE_URL="http://localhost:11434/v1"           # Ollama (ключ не нужен)
```

**Сохраните и закройте.** В nano: **Ctrl-O**, **Enter**, затем **Ctrl-X**. В TextEdit: **⌘S**, затем закройте окно.

**Загрузите настройки.** Откройте новое окно терминала или выполните:

```sh
source ~/.zshrc
```

**fish** использует другой синтаксис. Добавьте это в `~/.config/fish/config.fish`:

```fish
fish_add_path $HOME/.local/bin
set -gx ANTHROPIC_API_KEY "sk-ant-paste-your-key-here"
```

**Windows (PowerShell).** В Windows для этого не нужно редактировать файл профиля. Вместо этого сохраните ключ как пользовательскую переменную среды:

```powershell
setx ANTHROPIC_API_KEY "sk-ant-paste-your-key-here"
# или: setx OPENAI_API_KEY "sk-..."   /   setx OPENAI_BASE_URL "http://localhost:11434/v1"
```

Затем **закройте PowerShell и откройте новое окно**. `setx` не влияет на окно, в котором запущена. Это можно сделать и мышью: «Пуск» → *Edit environment variables for your account* → *New…*.

> 🔒 **Храните ключ в секрете.** Не добавляйте файл профиля в git и не вставляйте ключ в чаты. На Mac ключ можно хранить в Связке ключей; см. [Шаг 2 в разделе «Быстрый старт»](#шаг-2-подключите-модель).

### 4. Проверьте, что всё работает

```sh
agentiloop --version
```

Вы должны увидеть `agentiloop 0.0.1`. Теперь проверьте, что ключ загружен:

```sh
echo $ANTHROPIC_API_KEY | cut -c1-10    # macOS / Linux: должно вывести sk-ant-...
```
```powershell
$env:ANTHROPIC_API_KEY.Substring(0,10)  # Windows PowerShell
```

Если ничего не выводится, вернитесь к шагу 3. Ключ ещё не загружен.

### 5. Ваш первый сеанс

Перейдите в папку проекта и запустите полноэкранный интерфейс:

```sh
cd ~/my-project
agentiloop --tui
```

Используете **Ollama**? Укажите провайдера и скачанную модель: `agentiloop -p openai -m qwen2.5-coder --tui`.

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
agentiloop "what does src/main.rs do?"
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
| `command not found: agentiloop` | `~/.local/bin` нет в вашем PATH. Добавьте строку `export PATH=...` из шага 3, затем откройте новый терминал. В Windows откройте новое окно PowerShell |
| `Error: no provider credentials found` | Ключ не загружен. Повторите шаг 3, затем проверьте результат по шагу 4 |
| macOS: *"agentiloop" cannot be opened* / *unidentified developer* | Так бывает, если вы скачали файл через браузер, а не через `curl`. Выполните `xattr -d com.apple.quarantine ~/.local/bin/agentiloop` |
| Windows: *Windows protected your PC* | Нажмите **More info** → **Run anyway** |
| `401` / `invalid x-api-key` / ошибка аутентификации | Ключ неверный или содержит пробелы либо кавычки. Скопируйте его ещё раз и проверьте строку в профиле |
| Ollama: model not found | Выполните `ollama list` и передайте точное имя через `-m` |
| Программа продолжает использовать старую модель или провайдера | Она запоминает ваш последний выбор. Передайте `-p` / `-m`, чтобы изменить его, или удалите `~/.agentiloop/settings.json`, чтобы сбросить настройки |

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

AgentiLoop нужна модель, с которой можно общаться. Выберите одну из них:

| Я хочу использовать… | Что сделать |
|---|---|
| **Claude** (Anthropic) | `export ANTHROPIC_API_KEY=sk-ant-...` |
| **OpenAI** | `export OPENAI_API_KEY=sk-...` |
| **Ollama, LM Studio** или любой OpenAI-совместимый сервер | `export OPENAI_BASE_URL=http://localhost:11434/v1` (укажите адрес своего сервера; для локальных серверов ключ не нужен) |
| **oMLX** (локальные модели на Apple Silicon) | Обычно ничего. Запустите oMLX, затем запустите AgentiLoop с `-p omlx` (см. ниже) |

Для Claude можно использовать обычный API-ключ (`sk-ant-api…`) или токен Claude Code (`sk-ant-oat01-…`, который выдаёт `claude setup-token`). AgentiLoop сам определит, какой из них вы используете.

**Подробнее про oMLX.** Когда oMLX работает на том же Mac, AgentiLoop считывает порт сервера и API-ключ из собственного файла настроек oMLX (`~/.omlx/settings.json`), так что экспортировать ничего не нужно. Если oMLX работает на другой машине или вы хотите переопределить эти настройки, экспортируйте их сами:

```sh
export OMLX_BASE_URL=http://192.168.1.50:7777/v1   # адрес сервера oMLX (или OMLX_PORT=7777 для localhost)
export OMLX_API_KEY=...                            # API-ключ из настроек oMLX
```

Если в oMLX отключена проверка API-ключа, ключ не нужен.

`export` действует только в той вкладке терминала, где вы его ввели. Чтобы настройка сохранилась навсегда, добавьте эту строку в профиль оболочки (`~/.zshrc` на macOS). На Mac ключ можно хранить не в файле, а в Связке ключей:

```sh
# один раз: сохраните ключ в Связке ключей
security add-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w "sk-ant-..."

# в ~/.zshrc: загружать его в каждом новом терминале
export ANTHROPIC_API_KEY="$(security find-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w 2>/dev/null)"
```

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
- API-ключи: они остаются в вашем профиле оболочки

Чтобы сбросить всё, удалите `~/.agentiloop/settings.json`.

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
| `settings.json` | Запомненные провайдер, модели и параметры. Удалите его, чтобы сбросить настройки |
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
