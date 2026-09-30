<a href="https://fluxionai.world/register?source=github&campaign=aiagent&promo=AIAGENT"><img src="docs/sponsors/fluxion-ai-silver-ad_es.svg" width="900" alt="Fluxion AI, patrocinador Silver: una API unificada para GPT, Claude y otros modelos de IA líderes. Ahorra hasta un 70 % frente a los precios oficiales de la API y consigue $3 en créditos API." /></a>

# AgentiLoopCLI

🌐 [English](README.md) · [Español](README_es.md) · [Français](README_fr.md) · [Deutsch](README_de.md) · [中文 (简体)](README_zh.md) · [Русский](README_ru.md) · [한국어](README_ko.md) · [日本語](README_ja.md)

### 🎉 ¡Hemos publicado la versión v0.0.4 para Mac, Windows y Linux!

---

**¡Pruébalo!** Lee el README y mira cuánto tardas en tener AgentiLoop funcionando. Si te encuentras con algún problema, avísanos. Nos encantaría conocer tu opinión.

**Extra:** También hay una versión en Go: **AgentiLoopGo** → https://github.com/AgentiLoop/AgentiLoopGo

Prueba las dos y dinos cuál lo hace mejor: **¿Rust o Go?** 🦀 vs 🐹

---

### 💖 Patrocina AgentiLoop

¿Te gusta lo que ves? Ayúdanos a que AgentiLoop siga siendo rápido y multiplataforma. Patrocínanos en **[GitHub Sponsors → AgentiLoop](https://github.com/sponsors/AgentiLoop)**. Los niveles y las ventajas están en la [guía de patrocinio](https://github.com/AgentiLoop/Agent/blob/main/docs/SPONSORSHIP.md).

[![Patrocina AgentiLoop](https://img.shields.io/badge/Sponsor-AgentiLoop-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white)](https://github.com/sponsors/AgentiLoop)

---

AgentiLoop es un agente de programación con IA que se ejecuta en tu terminal, al estilo de Claude Code. Describes lo que quieres con tus propias palabras. El agente lee tus archivos, edita el código y ejecuta comandos para conseguirlo, y te pide permiso antes de cambiar nada.

Está escrito en Rust y funciona en macOS, Linux y Windows. Funciona con Claude (Anthropic), OpenAI, modelos locales a través de Ollama o LM Studio, y oMLX en Apple Silicon.

¡Creado con AgentiLoop Agent! Es nuestro bebé. Tienes binarios precompilados para macOS, Linux y Windows en la página de [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases), o puedes compilarlo desde el código fuente con Rust.

<img src="docs/pong.png" width="900" alt="AgentiLoop escribe, compila y lanza un juego de Pong al estilo Atari en SwiftUI a partir de un solo mensaje, con el diff en vivo y la ventana del juego en marcha" />

---

## ⚡ Primera ejecución: el asistente de configuración lo hace todo

No hay nada que configurar a mano. La primera vez que ejecutas `agentiloop`, un asistente de configuración integrado arranca solo. Te pregunta qué proveedor quieres (Claude, OpenAI, Ollama / LM Studio u oMLX), toma tu clave de API (escrita de forma oculta), comprueba que la clave funciona, te deja elegir un modelo y lo guarda todo en `~/.agentiloop`. Alrededor de un minuto, sin archivos de configuración, sin líneas `export`.

<img src="docs/setup-wizard-tui.png" width="900" alt="El asistente de configuración dentro de la interfaz a pantalla completa (TUI): proveedor, clave de API oculta, comprobación de conexión, lista de modelos, dónde guardar la clave y el primer mensaje" />

Vuelve a ejecutarlo cuando quieras con `agentiloop --setup` (añade `--tui` para la versión a pantalla completa, o escribe `/setup` dentro de una sesión). `agentiloop --reset` lo olvida todo y empieza de cero. ¿Nunca has instalado un programa de línea de comandos? Sigue [¿Nuevo aquí?](#-nuevo-aquí-en-marcha-en-5-minutos) más abajo, paso a paso.

---

## 🚀 ¿Nuevo aquí? En marcha en 5 minutos

Sin Rust, sin Go, sin compilar. Descargas un archivo, le das una clave de API y empiezas a chatear. Sigue los pasos en orden.

### 1. Descarga AgentiLoop

Primero averigua qué archivo necesitas:

| Tu ordenador | Archivo que descargar |
|---|---|
| Mac con Apple Silicon (M1, M2, M3, M4…) | `agentiloop-macos-arm64.tar.gz` |
| Mac con chip Intel | `agentiloop-macos-x86_64.tar.gz` |
| Linux, PC de 64 bits | `agentiloop-linux-x86_64.tar.gz` |
| Linux en ARM (Raspberry Pi 4/5, servidores ARM) | `agentiloop-linux-arm64.tar.gz` |
| Windows 10/11 | `agentiloop-windows-x86_64.zip` |

¿No estás seguro? En Mac o Linux, ejecuta `uname -m`. `arm64` o `aarch64` significa **arm64**, y `x86_64` significa **x86_64**.

**macOS y Linux.** Abre Terminal y pega estas líneas. Este ejemplo usa el archivo de Apple Silicon, así que cambia `macos-arm64` en las tres primeras líneas si el tuyo es distinto:

```sh
curl -LO https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.4/agentiloop-macos-arm64.tar.gz
tar xzf agentiloop-macos-arm64.tar.gz
mkdir -p ~/.local/bin && mv agentiloop-macos-arm64/agentiloop ~/.local/bin/
```

Eso coloca el programa en `~/.local/bin`, una carpeta dentro de tu directorio personal. El asistente de configuración del paso 3 se ofrece a decirle a tu terminal que busque ahí.

**Windows.** Abre **PowerShell** (menú Inicio → escribe "PowerShell") y pega:

```powershell
Invoke-WebRequest https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.4/agentiloop-windows-x86_64.zip -OutFile agentiloop.zip
Expand-Archive agentiloop.zip -DestinationPath $HOME\agentiloop -Force
$p = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$p;$HOME\agentiloop\agentiloop-windows-x86_64", "User")
```

Las dos últimas líneas añaden AgentiLoop a tu PATH. **Cierra PowerShell y abre una ventana nueva** para que se aplique el cambio.

### 2. Consigue una clave de API

AgentiLoop es el agente. El "cerebro" es un modelo de IA al que lo conectas. Elige **uno**:

| Opción | Dónde conseguirla | Coste |
|---|---|---|
| **Claude** (recomendado) | [console.anthropic.com](https://console.anthropic.com/settings/keys) → *Create Key*. Empieza por `sk-ant-` | Pago por uso |
| **OpenAI** | [platform.openai.com/api-keys](https://platform.openai.com/api-keys). Empieza por `sk-` | Pago por uso |
| **Ollama** (se ejecuta en tu propio ordenador) | Instálalo desde [ollama.com](https://ollama.com) y luego ejecuta `ollama pull qwen2.5-coder` | Gratis, sin clave |

Guarda la clave en un lugar seguro. La pegarás en el siguiente paso.

### 3. Ejecuta el asistente de configuración

No editas ningún archivo ni escribes ningún comando `export`. AgentiLoop tiene un asistente de configuración integrado que te hace unas pocas preguntas y guarda la clave por ti.

Como `~/.local/bin` todavía no está en tu PATH, esta única vez arráncalo con su ruta completa (en Windows el paso 1 ya arregló el PATH, así que escribe simplemente `agentiloop`):

```sh
~/.local/bin/agentiloop
```

La primera vez, cuando todavía no hay ninguna clave configurada, el asistente arranca solo. Tarda alrededor de un minuto y pregunta cinco cosas:

1. **Qué proveedor** — Claude, OpenAI, un servidor local compatible con OpenAI (Ollama, LM Studio, …) u oMLX. Escribe un número.
2. **Tu clave de API** — se escribe de forma oculta, no se muestra nada en pantalla. Los servidores locales normalmente no necesitan ninguna; con oMLX en el mismo Mac, el asistente lee la clave de la propia configuración de oMLX, así que ni siquiera la pide.
3. **Comprobación de conexión** — el asistente habla con el proveedor al momento. Si la clave es incorrecta te lo dice y te ofrece intentarlo de nuevo; no se guarda nada hasta que funciona.
4. **Qué modelo** — elige uno de la lista que devuelve el proveedor, o pulsa Enter para el predeterminado. Puedes cambiarlo en cualquier momento más adelante con `/model`.
5. **Dónde guardar la clave** — pulsa Enter para la opción predeterminada, `~/.agentiloop/env`, un archivo privado que solo lee AgentiLoop. (Las otras opciones, para quien también quiera la clave en su shell o en el Llavero de macOS, se explican en [Avanzado: definir la clave a mano](#paso-2-conecta-un-modelo) en el Inicio rápido.)

Por último, el asistente se da cuenta de que `~/.local/bin` no está en tu PATH y se ofrece a añadirlo. Pulsa **Enter** (sí). Después dice `All set` y te deja en el prompt. Aquí tienes una ejecución completa en el terminal normal, eligiendo Claude y aceptando los valores predeterminados (tu lista de modelos será distinta):

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

Puedes empezar a chatear aquí mismo, o escribir `/exit` y continuar con el paso 4. A partir de ahora, un simple `agentiloop` arranca directamente en el prompt.

Para Claude puedes pegar una clave de API normal (`sk-ant-api…`) o un token de Claude Code (`sk-ant-oat01-…`, de `claude setup-token`); AgentiLoop detecta de qué tipo es. Si elegiste la opción 3 (Ollama, LM Studio, …), el asistente también te pide la URL del servidor y ofrece `http://localhost:11434/v1` como valor predeterminado, así que para un Ollama local basta con pulsar Enter.

**El mismo asistente, a pantalla completa.** El asistente funciona en la interfaz que uses. Arráncalo con `--tui` y las mismas preguntas aparecen dentro de la interfaz a pantalla completa; cuando termina ya estás en el prompt:

<img src="docs/setup-wizard-tui.png" width="900" alt="El asistente de configuración dentro de la interfaz a pantalla completa (TUI): proveedor, clave de API oculta, comprobación de conexión, lista de modelos, dónde guardar la clave y el primer mensaje" />

**Vuelve a ejecutarlo o rehazlo cuando quieras:**

```bash
agentiloop --setup          # asistente en el terminal normal
agentiloop --setup --tui    # asistente dentro de la TUI a pantalla completa (como en la captura)
/setup                      # desde dentro de una sesión en marcha (REPL o TUI)
agentiloop --reset          # olvidarlo todo y empezar de cero
```

> 🔒 **Mantén tu clave en privado.** `~/.agentiloop/env` solo lo puedes leer tú. No pegues la clave en chats ni la subas a git. En un Mac, la opción 3 de la pregunta «Where should the credential be saved?» la guarda en el Llavero, así que nunca está en el disco en texto plano.

¿Prefieres gestionar la clave tú mismo con variables de entorno? También funciona, pero es la vía avanzada; consulta [Avanzado: definir la clave a mano](#paso-2-conecta-un-modelo) en el Inicio rápido.


### 4. Comprueba que funciona

**Abre una ventana nueva de terminal** para que recoja el cambio del PATH hecho por el asistente (Windows: una ventana nueva de PowerShell). Después:

```sh
agentiloop --version
```

Deberías ver `agentiloop 0.0.4`. Ahora ejecútalo sin opciones:

```sh
agentiloop
```

Debería ir directamente al prompt. Si en su lugar vuelve a arrancar el asistente, la clave no se guardó: repite el paso 3.

### 5. Tu primera sesión

Ve a una carpeta de proyecto y abre la interfaz a pantalla completa:

Empieza con una carpeta de prueba nueva y vacía para probarlo sin riesgos. Para un proyecto real, entra con `cd` en la carpeta de ese proyecto (por ejemplo `cd ~/code/my-app`).

```sh
mkdir -p ~/agentiloop-test
cd ~/agentiloop-test
agentiloop --tui
```

¿Usas **Ollama**? El asistente ya recordó el servidor y el modelo que elegiste. Para cambiar a otro modelo que hayas descargado, escribe `/model` dentro de la sesión.

Ahora simplemente escribe lo que quieres con tus propias palabras y pulsa **Enter**. Algunas buenas primeras peticiones:

```text
explica qué hace este proyecto
lista los archivos de src y dime cuál es el punto de entrada
busca los comentarios TODO y resúmelos
añade una opción --verbose al analizador de la línea de comandos
ejecuta los tests y arregla lo que falle
crea un README.md para este proyecto
```

Antes de que el agente cambie un archivo o ejecute un comando, te pregunta. Pulsa **y** para sí, **n** para no, **a** para permitir siempre esa herramienta durante la sesión, o **Esc** para saltarte el paso. Pulsa **Ctrl-C** para salir. La próxima vez, un simple `agentiloop` arranca igual y retoma tu última conversación.

¿Solo quieres una respuesta sin chat? Pasa la pregunta como argumento:

```sh
agentiloop "explain what this project does"
```

### ¿Qué puede hacer? (herramientas)

El agente trabaja con siete herramientas integradas. No las llamas tú. Describes el objetivo y el agente elige la herramienta:

| Herramienta | Qué hace | ¿Pregunta antes? |
|---|---|---|
| `read_file` | Lee un archivo (con números de línea) | No |
| `list_dir` | Lista los archivos de una carpeta | No |
| `glob` | Busca archivos por patrón de nombre (`*.rs`, `src/**/*.go`) | No |
| `grep` | Busca dentro de los archivos con una expresión regular | No |
| `write_file` | Crea un archivo nuevo o sobrescribe uno | **Sí** |
| `edit_file` | Cambia un fragmento exacto de texto en un archivo | **Sí** |
| `bash` | Ejecuta un comando de shell, como tests, compilaciones o `git` (`sh -c` en Mac/Linux, `cmd /C` en Windows) | **Sí** |

¿Quieres más herramientas, como búsqueda web, bases de datos o GitHub? Añade servidores MCP; consulta [Añadir herramientas con MCP](#añadir-herramientas-con-mcp-opcional).

### El comando de ayuda

`agentiloop --help` muestra todas las opciones:

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

Dentro de una sesión, escribe `/help` para ver los comandos del chat (`/model`, `/sessions`, `/resume`, `/clear`, `/compact`, `/mcp`, `/exit`). La referencia completa está en [Todas las opciones](#todas-las-opciones) y [Comandos dentro del chat](#comandos-dentro-del-chat).

### ¿Atascado? Soluciones rápidas

| Ves | Solución |
|---|---|
| `command not found: agentiloop` | `~/.local/bin` no está en tu PATH. Abre primero una ventana nueva de terminal; si eso no ayuda, ejecuta `~/.local/bin/agentiloop --setup` y di que sí cuando ofrezca añadirlo al PATH. En Windows, abre una ventana nueva de PowerShell |
| `Error: no provider credentials found` | No hay ninguna clave guardada. Ejecuta `agentiloop --setup` (paso 3) y compruébalo con el paso 4 |
| macOS: *"agentiloop" cannot be opened* / *unidentified developer* | Esto pasa si lo descargaste con un navegador en lugar de con `curl`. Ejecuta `xattr -d com.apple.quarantine ~/.local/bin/agentiloop` |
| Windows: *Windows protected your PC* | Haz clic en **More info** → **Run anyway** |
| `401` / `invalid x-api-key` / error de autenticación | La clave es incorrecta o se pegó con espacios o comillas. Cópiala de nuevo y ejecuta `agentiloop --setup` para introducirla otra vez |
| Ollama: modelo no encontrado | Ejecuta `ollama list` y pasa el nombre exacto con `-m` |
| Sigue usando un modelo o proveedor antiguo | Recuerda tus últimas elecciones. Pasa `-p` / `-m` para cambiarlas, o ejecuta `agentiloop --reset` para empezar de cero |

¿Sigues atascado? [Abre un issue](https://github.com/AgentiLoop/AgentiLoopCLI/issues) y pega el comando y el error. Te ayudaremos.

---

## Inicio rápido

Tres pasos: instálalo, dale un modelo y ejecútalo.

### Paso 1: Instalar

**Descarga:** baja el archivo comprimido para tu plataforma desde [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases), descomprímelo y pon `agentiloop` (`agentiloop.exe` en Windows) en tu PATH.

**O compílalo:** si todavía no tienes Rust, instálalo desde [rustup.rs](https://rustup.rs). Después:

```sh
git clone https://github.com/AgentiLoop/AgentiLoopCLI.git
cd AgentiLoopCLI
cargo install --path crates/agentiloop-cli
```

Esto compila el programa y deja un comando `agentiloop` en tu PATH, en `~/.cargo/bin`.

> **¿No quieres instalarlo?** Todo lo de este README también funciona desde dentro de la carpeta del repositorio. Donde veas `agentiloop <options>`, escribe `cargo run -- <options>` en su lugar. Todo lo que va después de `--` se le pasa a AgentiLoop.

### Paso 2: Conecta un modelo

AgentiLoop necesita un modelo con el que hablar. **No tienes que definir ninguna variable de entorno para esto**: simplemente ejecuta `agentiloop` y el asistente de configuración integrado te hace unas pocas preguntas y lo guarda todo por ti. La guía paso a paso, con una ejecución completa, está más arriba en [3. Ejecuta el asistente de configuración](#3-ejecuta-el-asistente-de-configuración).

<details>
<summary><b>Avanzado: definir la clave a mano</b> (sáltate esto si el asistente te ha funcionado)</summary>

Si prefieres gestionar la clave tú mismo, o ejecutas AgentiLoop en un script o en CI donde nadie puede responder al asistente, define una de estas variables de entorno y AgentiLoop la usará sin preguntar:

| Quiero usar… | Define esto |
|---|---|
| **Claude** (Anthropic) | `export ANTHROPIC_API_KEY=sk-ant-...` |
| **OpenAI** | `export OPENAI_API_KEY=sk-...` |
| **Ollama, LM Studio** o cualquier servidor compatible con OpenAI | `export OPENAI_BASE_URL=http://localhost:11434/v1` (la dirección de tu servidor; los servidores locales no necesitan clave) |
| **oMLX** (modelos locales en Apple Silicon) | Normalmente nada. Inicia oMLX y luego ejecuta AgentiLoop con `-p omlx` |

**Detalles de oMLX.** Cuando oMLX se ejecuta en el mismo Mac, AgentiLoop lee el puerto del servidor y la clave de API del propio archivo de configuración de oMLX (`~/.omlx/settings.json`). Si oMLX se ejecuta en otra máquina, o quieres sobrescribir esa configuración:

```sh
export OMLX_BASE_URL=http://192.168.1.50:7777/v1   # la dirección del servidor oMLX (o OMLX_PORT=7777 para localhost)
export OMLX_API_KEY=...                            # la clave de API de la configuración de oMLX
```

Si oMLX tiene desactivada la verificación de la clave de API, no hace falta ninguna clave.

Un `export` solo dura en la pestaña de terminal donde lo escribiste. Para que sea permanente añadirías la línea a tu perfil de shell (`~/.zshrc` en macOS), que es exactamente lo que hace por ti la opción **"Also add it to ~/.zshrc"** del asistente. Del mismo modo, la opción **"macOS Keychain"** del asistente es la versión automática de esto:

```sh
# una sola vez: guarda la clave en tu Llavero
security add-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w "sk-ant-..."

# en ~/.zshrc: cárgala en cada terminal nueva
export ANTHROPIC_API_KEY="$(security find-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w 2>/dev/null)"
```

</details>

### Paso 3: Ejecútalo

Ve al proyecto en el que quieres trabajar y arranca AgentiLoop:

```sh
cd ~/my-project
agentiloop --tui
```

`--tui` abre la interfaz a pantalla completa, que es la que te recomendamos. Escribe lo que quieres, por ejemplo *"busca dónde se carga el archivo de configuración y añade una opción --verbose"*, y pulsa Enter.

Verás las respuestas del agente, cada herramienta que usa (🔧) y cada resultado (✓ o ✖). El recuadro de abajo muestra lo que está haciendo en ese momento, por ejemplo ` ✻ Thinking...  12s `. Antes de escribir un archivo o ejecutar un comando, te pregunta:

- **y**: sí, esta vez
- **n**: no
- **a**: permitir siempre esta herramienta durante el resto de la sesión
- **Esc**: saltar este paso, pero seguir adelante

---

## Tres formas de usarlo

| Modo | Comando | Ideal para |
|---|---|---|
| **TUI** (pantalla completa) | `agentiloop --tui` | El día a día: historial con desplazamiento, estado en vivo, enlaces en los que se puede hacer clic |
| **Chat** (línea a línea) | `agentiloop` | Terminales sencillas, o si prefieres texto plano |
| **Una sola vez** | `agentiloop "explain this project"` | Una pregunta: responde y termina. Práctico en scripts |

Teclas en la TUI: **Enter** envía · **↑ / ↓** recorren las peticiones anteriores · **PgUp / PgDn** o la rueda del ratón desplazan · **Ctrl-U** borra la línea · **Ctrl-C** sale.

---

## Recuerda tu configuración

Solo escribes tus opciones una vez. AgentiLoop guarda cómo lo lanzaste, así que la próxima vez un simple `agentiloop` arranca igual:

```sh
agentiloop -p anthropic --tui    # la primera vez: elige proveedor y TUI
agentiloop                       # a partir de ahora: mismo proveedor, mismo modelo, TUI y tu última conversación
```

Lo que recuerda:

- **Proveedor** (`-p`) y **TUI activada/desactivada** (`--tui` / `--no-tui`)
- **Modelo**: el último que usaste, por separado para cada proveedor. Al volver a un proveedor, vuelve también su modelo.
- **Límites**: `--max-turns` y `--compact-at`
- **Tu conversación**: retoma la última conversación de la carpeta actual, si esa conversación usaba el mismo proveedor. Los mensajes anteriores se vuelven a mostrar en pantalla, así que puedes desplazarte hacia atrás y ver dónde lo dejaste

Para cambiar algo, pasa la nueva opción. Se aplica al momento y se recuerda desde entonces:

```sh
agentiloop -p omlx        # cambia a oMLX (también vuelve su último modelo usado)
agentiloop -m <model>     # cambia de modelo
agentiloop --no-tui       # vuelve al chat línea a línea
agentiloop --new          # empieza una conversación nueva (la anterior sigue guardada)
```

Algunas cosas **nunca** se recuerdan, a propósito:

- `--yes`: saltarse las peticiones de permiso tiene que ser una decisión deliberada cada vez
- `--no-mcp`, `-C` y las peticiones de una sola vez
- Las claves de API: esas viven en `~/.agentiloop/env` (escrito por el asistente) o en el entorno de tu shell, nunca en `settings.json`

Para olvidarlo todo, ejecuta `agentiloop --reset`.

---

## Todas las opciones

Cada opción también se puede configurar con una variable de entorno, que aparece en la segunda columna. Una opción que escribes siempre tiene prioridad sobre un valor recordado.

| Opción | Variable de entorno | Qué hace |
|---|---|---|
| `-p, --provider <name>` | `AGENTILOOP_PROVIDER` | `anthropic`, `openai` u `omlx`. Si no indicas ninguno, AgentiLoop usa el último, o lo detecta a partir de tus claves (primero Anthropic, luego OpenAI, luego oMLX) |
| `-m, --model <id>` | `AGENTILOOP_MODEL` | Qué modelo usar |
| `--tui` / `--no-tui` | `AGENTILOOP_TUI` | Interfaz a pantalla completa activada / desactivada |
| `--new` | | Empieza una conversación nueva en lugar de continuar |
| `-c, --continue` | | Continúa aquí la última conversación (ya es lo predeterminado) |
| `-r, --resume <id>` | | Vuelve a abrir una conversación concreta (encuentra los ids con `/sessions`) |
| `-C, --cwd <folder>` | | Trabaja en una carpeta distinta de la que estás |
| `--yes` | `AGENTILOOP_YES` | No pregunta antes de ejecutar herramientas. ⚠️ Solo para uso automatizado y de confianza |
| `--no-mcp` | `AGENTILOOP_NO_MCP` | No inicia servidores MCP (mira más abajo) |
| `--setup` | | Vuelve a ejecutar el asistente de primera configuración (proveedor, clave, modelo). Combínalo con `--tui` para ejecutarlo dentro de la interfaz a pantalla completa |
| `--reset` | | Como nuevo: borra `~/.agentiloop`, el bloque de agentiloop en tu perfil de shell y los elementos del Llavero que creó el asistente (en Windows: las variables de entorno de usuario que definió). Las líneas `export` escritas a mano solo se comentan, y solo si dices que sí. Añade `--yes` para saltarte las preguntas |
| `--max-turns <n>` | | Máximo de pasos que puede dar el agente por petición (50 por defecto) |
| `--compact-at <tokens>` | `AGENTILOOP_COMPACT_AT` | Cuándo resumir una conversación larga (150000 por defecto, `0` = nunca) |
| `-h` / `-V` | | Ayuda / versión |

Algunos ejemplos:

```sh
agentiloop -p openai -m gpt-4o-mini "summarize this repo"    # una pregunta con un modelo concreto
agentiloop -C ../other-repo --tui                            # trabaja en otro proyecto
agentiloop --yes "run the tests and fix any failures"        # sin supervisión, sin preguntas
```

**¿Qué modelo se usa?** Gana el primero que se cumpla:

1. `-m` en la línea de comandos
2. el modelo de la conversación que estás continuando
3. el último modelo que usaste con este proveedor
4. el predeterminado del proveedor: `claude-sonnet-5` para Anthropic, `gpt-4o-mini` para OpenAI, o el primer modelo que ofrezca oMLX

---

## Comandos dentro del chat

Escríbelos en el prompt, en la TUI o en el chat:

| Comando | Qué hace |
|---|---|
| `/model` | Muestra los modelos disponibles. `/model 3` o `/model <id>` cambia de modelo (y se recuerda) |
| `/sessions` | Lista tus conversaciones guardadas, de la más reciente a la más antigua |
| `/resume <n or id>` | Vuelve a abrir una de ellas |
| `/clear` | Borra la conversación y empieza una nueva |
| `/compact` | Resume la conversación ahora para liberar espacio |
| `/mcp` | Muestra los servidores MCP conectados y sus herramientas |
| `/help` | Lista estos comandos |
| `/exit` | Salir |

---

## Conversaciones largas

Los modelos solo pueden leer una cantidad limitada de una vez. Cuando una conversación crece mucho (por defecto, cuando una petición llega a 150.000 tokens), AgentiLoop le pide al modelo que la resuma hasta ese punto y continúa a partir del resumen. Verás una nota 📦 cuando eso ocurra. `/compact` lo hace cuando quieras, y `--compact-at 0` lo desactiva.

---

## Instrucciones del proyecto

Pon un archivo `AGENTS.md` (o `CLAUDE.md`) en tu proyecto y AgentiLoop lo lee al arrancar y lo sigue: comandos de compilación, estilo de código, cosas que evitar. Lo busca en la carpeta actual y luego en las carpetas superiores, hasta la raíz del proyecto (la carpeta con `.git`). Un `~/.agentiloop/AGENTS.md` personal se aplica a todos los proyectos; el archivo del proyecto va después y tiene prioridad. Los archivos se recortan a 32 KB. Una línea `instructions: <ruta>` muestra qué archivos se cargaron.

---

## Añadir herramientas con MCP (opcional)

Los servidores [MCP](https://modelcontextprotocol.io) le dan al agente herramientas extra, como acceso a bases de datos, búsqueda web o tus propios scripts. Enuméralos en un archivo JSON:

- `~/.agentiloop/mcp.json`: disponible en todos los proyectos
- `.mcp.json` en una carpeta de proyecto: solo en ese proyecto. Si un nombre aparece en ambos archivos, gana este.

El formato es el mismo que usan Claude Code, Claude Desktop y Agent!, así que puedes copiar configuraciones que ya tengas:

```json
{ "mcpServers": {
    "Local":  { "command": "my-mcp-server", "args": ["--flag"], "env": { "API_KEY": "${MY_KEY}" } },
    "Remote": { "url": "https://example.com/mcp", "headers": { "Authorization": "Bearer ${TOKEN}" } }
} }
```

Cómo funciona:

- **Dos tipos de servidor.** Un servidor con `command` es un programa local que AgentiLoop inicia por ti. A un servidor con `url` se accede por HTTP. Funcionan tanto los servidores más nuevos "Streamable HTTP" como los más antiguos "SSE"; una URL que termina en `/sse` (o `"transport": "sse"`) selecciona el estilo antiguo.
- **Nombres de herramientas.** Cada herramienta del servidor le aparece al agente como `mcp_<server>_<tool>`, por ejemplo `mcp_Local_search`.
- **Secretos.** `${VAR}` (o `${VAR:-default}`) se rellena desde tu entorno, así que las claves no tienen que estar en el archivo.
- **Permisos.** Las herramientas MCP piden permiso como cualquier otra herramienta, a menos que el servidor marque una herramienta como de solo lectura.
- **Desactivar servidores.** Añade `"disabled": true` para saltarte un servidor, o ejecuta con `--no-mcp` para saltártelos todos.
- **Seguridad.** `http://` sin cifrar solo se permite para localhost; los servidores remotos necesitan `https://`.

Escribe `/mcp` para ver qué servidores se han conectado, sus herramientas y cualquier error.

---

## Dónde se guardan las cosas

Todo vive en `~/.agentiloop/`. Define `AGENTILOOP_HOME` para usar otra carpeta, por ejemplo un perfil de pruebas separado.

| Archivo | Qué contiene |
|---|---|
| `settings.json` | El proveedor, los modelos y las opciones recordados, más lo que el asistente escribió en otros sitios |
| `env` | Tu clave, escrita por el asistente (`KEY=value`, modo de archivo 600). Se carga al arrancar; un `export` en tu shell tiene prioridad |
| `sessions/` | Tus conversaciones, un archivo por cada una |
| `mcp.json` | Tus servidores MCP |
| `history.txt` | Las peticiones que has escrito (para ↑ / ↓) |

---

## Otras variables de entorno

Rara vez las necesitarás:

| Variable | Uso |
|---|---|
| `ANTHROPIC_BASE_URL` | Envía las peticiones de Anthropic a un proxy o a un servidor compatible |
| `ANTHROPIC_OAUTH_TOKEN` | Alternativa a `ANTHROPIC_API_KEY` para un token de Claude Code |
| `OMLX_BASE_URL`, `OMLX_PORT`, `OMLX_API_KEY` | Dirección y clave del servidor oMLX. Tienen prioridad sobre `~/.omlx/settings.json`, que se lee por defecto (puerto 8000 si no se define ninguno) |
| `RUST_LOG=debug` | Muestra los registros de depuración, incluido el uso de tokens por petición |

---

## Para desarrolladores

### Compilar y probar

```sh
cargo build --release     # → target/release/agentiloop
cargo test --workspace    # funciona sin conexión, no necesita claves de API
```

Los tests no tocan la red. El bucle del agente se ejecuta contra un modelo falso con guion, y los analizadores de streaming contra un servidor de pruebas local. El cliente MCP se prueba contra un servidor de ejemplo incluido, con los tres tipos de conexión (stdio, HTTP, SSE). Puedes ejecutar ese servidor tú mismo para probar MCP a mano:

```sh
cargo run -p agentiloop-mcp --example mcp-example-server -- --http 8791   # o --sse 8792, o --stdio
```

### Cómo está organizado el código

El proyecto está dividido en cinco crates, y cada uno se apoya en los anteriores:

| Crate | Qué contiene |
|---|---|
| `agentiloop-core` | El corazón: el bucle del agente, los mensajes, las interfaces de herramientas y proveedores, los permisos, las sesiones, los resúmenes |
| `agentiloop-provider` | Habla con los modelos: Anthropic, servidores compatibles con OpenAI, oMLX |
| `agentiloop-tools` | Herramientas integradas: `read_file`, `write_file`, `edit_file`, `list_dir`, `glob`, `grep`, `bash` |
| `agentiloop-mcp` | El cliente MCP, portado desde AgentMCP de Agent! en Swift |
| `agentiloop-cli` | El programa `agentiloop`: opciones, chat, TUI, configuración |

### Dependencias

Mantenemos pocas dependencias: 18 crates externos, y cada crate enumera solo los que realmente usa.

| Crates | Para qué se usan |
|---|---|
| `tokio`, `futures`, `async-trait` | Ejecutar cosas de forma concurrente |
| `serde`, `serde_json` | Leer y escribir JSON |
| `reqwest` | Peticiones HTTP a los modelos y a los servidores MCP |
| `anyhow`, `thiserror` | Gestión de errores |
| `tracing`, `tracing-subscriber` | Registros (logging) |
| `dirs` | Encontrar tu carpeta personal |
| `clap` | Opciones de la línea de comandos |
| `rustyline` | El chat línea a línea |
| `ratatui`, `unicode-width`, `textwrap` | La TUI |
| `pulldown-cmark`, `syntect` | Markdown y resaltado de código |

---

## Hoja de ruta

- [x] Respuestas en streaming
- [x] Proveedores compatibles con OpenAI
- [x] Resumen de conversaciones largas
- [x] Conversaciones guardadas
- [x] TUI a pantalla completa
- [x] Cliente MCP
- [ ] ¿Qué viene después (What's NeXT)?

## Licencia

[PolyForm Noncommercial 1.0.0](LICENSE). Puedes usar, modificar y compartir este software con fines personales y no comerciales. El uso comercial, incluida la creación o venta de versiones comerciales, está reservado a AgentiLoop. Contacta con AgentiLoop para obtener una licencia comercial.

---

<a href="https://fluxionai.world/register?source=github&campaign=aiagent&promo=AIAGENT"><img src="docs/sponsors/fluxion-ai-silver-ad_es.svg" width="900" alt="Fluxion AI, patrocinador Silver: una API unificada para GPT, Claude y otros modelos de IA líderes. Ahorra hasta un 70 % frente a los precios oficiales de la API y consigue $3 en créditos API." /></a>

---

**AgentiLoop:** [agentiloop.ai](https://agentiloop.ai/)

Copyright © 2026 AgentiLoop.ai, a Logos InkPen LLC company. All rights reserved.
