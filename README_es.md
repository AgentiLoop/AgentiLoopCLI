# AgentiLoopCLI

🌐 [English](README.md) · [Español](README_es.md) · [Français](README_fr.md) · [Deutsch](README_de.md) · [中文 (简体)](README_zh.md) · [Русский](README_ru.md) · [한국어](README_ko.md) · [日本語](README_ja.md)

### ¡Hemos publicado una versión preliminar v0.0.1 para Mac, Windows y Linux!

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

<img width="2048" height="1152" alt="AgentiLoop programando en acción" src="https://github.com/user-attachments/assets/d910bbd2-b47d-4c4a-89af-ac0753ccf279" />

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
curl -LO https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.1/agentiloop-macos-arm64.tar.gz
tar xzf agentiloop-macos-arm64.tar.gz
mkdir -p ~/.local/bin && mv agentiloop-macos-arm64/agentiloop ~/.local/bin/
```

Eso coloca el programa en `~/.local/bin`, una carpeta dentro de tu directorio personal. En el paso 3 le dirás a tu terminal que busque ahí.

**Windows.** Abre **PowerShell** (menú Inicio → escribe "PowerShell") y pega:

```powershell
Invoke-WebRequest https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.1/agentiloop-windows-x86_64.zip -OutFile agentiloop.zip
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

### 3. Guarda tu configuración en el perfil de tu shell

Tu **perfil de shell** es un pequeño archivo de texto que tu terminal lee cada vez que abre una ventana nueva. Pon ahí tu configuración y solo tendrás que hacerlo una vez. Si te lo saltas, tendrás que escribir tu clave de nuevo en cada terminal nueva. Es el motivo n.º 1 por el que la gente se queda atascada.

**¿Qué archivo es?**

| Sistema | Shell (por defecto) | Archivo de perfil |
|---|---|---|
| macOS (Catalina 10.15 y posteriores) | zsh | `~/.zshrc` |
| La mayoría de distros Linux | bash | `~/.bashrc` |
| Linux o Mac con zsh | zsh | `~/.zshrc` |
| Shell fish | fish | `~/.config/fish/config.fish` |
| Windows | PowerShell | no hace falta, mira más abajo |

¿No sabes qué shell usas? Ejecuta `echo $SHELL`. `~` significa tu carpeta personal, así que `~/.zshrc` es, por ejemplo, `/Users/you/.zshrc`. Los archivos que empiezan por un punto están ocultos en Finder y en los exploradores de archivos, y eso es normal.

**Abre el archivo.** Usa uno de estos comandos (crean el archivo si todavía no existe):

```sh
nano ~/.zshrc                        # funciona en todas partes, directamente en la terminal
touch ~/.zshrc && open -e ~/.zshrc   # macOS: lo abre en TextEdit
```

(Linux con bash: usa `~/.bashrc` en lugar de `~/.zshrc`.)

**Añade estas líneas al final.** Deja solo la línea de la clave que necesites y pega tu clave real entre las comillas:

```sh
# AgentiLoop
export PATH="$HOME/.local/bin:$PATH"

export ANTHROPIC_API_KEY="sk-ant-paste-your-key-here"          # Claude
# export OPENAI_API_KEY="sk-paste-your-key-here"               # OpenAI
# export OPENAI_BASE_URL="http://localhost:11434/v1"           # Ollama (no hace falta clave)
```

**Guarda y cierra.** En nano: **Ctrl-O**, **Enter** y luego **Ctrl-X**. En TextEdit: **⌘S** y luego cierra la ventana.

**Cárgalo.** Abre una ventana nueva de terminal o ejecuta:

```sh
source ~/.zshrc
```

**fish** usa una sintaxis distinta. Pon esto en `~/.config/fish/config.fish`:

```fish
fish_add_path $HOME/.local/bin
set -gx ANTHROPIC_API_KEY "sk-ant-paste-your-key-here"
```

**Windows (PowerShell).** Windows no tiene un archivo de perfil que editar para esto. En su lugar, guarda la clave como variable de entorno de usuario:

```powershell
setx ANTHROPIC_API_KEY "sk-ant-paste-your-key-here"
# o bien: setx OPENAI_API_KEY "sk-..."   /   setx OPENAI_BASE_URL "http://localhost:11434/v1"
```

Después **cierra PowerShell y abre una ventana nueva**. `setx` no afecta a la ventana en la que se ejecuta. También puedes hacerlo con el ratón: Inicio → *Edit environment variables for your account* → *New…*.

> 🔒 **Mantén tu clave en privado.** No subas tu archivo de perfil a git ni pegues la clave en chats. En un Mac puedes guardarla en el Llavero; consulta el [Paso 2 del Inicio rápido](#paso-2-conecta-un-modelo).

### 4. Comprueba que funciona

```sh
agentiloop --version
```

Deberías ver `agentiloop 0.0.1`. Ahora comprueba que la clave está cargada:

```sh
echo $ANTHROPIC_API_KEY | cut -c1-10    # macOS / Linux: debería mostrar sk-ant-...
```
```powershell
$env:ANTHROPIC_API_KEY.Substring(0,10)  # Windows PowerShell
```

Si no muestra nada, vuelve al paso 3. La clave todavía no está cargada.

### 5. Tu primera sesión

Ve a una carpeta de proyecto y abre la interfaz a pantalla completa:

```sh
cd ~/my-project
agentiloop --tui
```

¿Usas **Ollama**? Indícale el proveedor y un modelo que hayas descargado: `agentiloop -p openai -m qwen2.5-coder --tui`.

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
agentiloop "what does src/main.rs do?"
```

### ¿Qué puede hacer? (herramientas)

El agente trabaja con cinco herramientas integradas. No las llamas tú. Describes el objetivo y el agente elige la herramienta:

| Herramienta | Qué hace | ¿Pregunta antes? |
|---|---|---|
| `read_file` | Lee un archivo (con números de línea) | No |
| `list_dir` | Lista los archivos de una carpeta | No |
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
| `command not found: agentiloop` | `~/.local/bin` no está en tu PATH. Añade la línea `export PATH=...` del paso 3 y abre una terminal nueva. En Windows, abre una ventana nueva de PowerShell |
| `Error: no provider credentials found` | No hay ninguna clave cargada. Repite el paso 3 y compruébalo con el paso 4 |
| macOS: *"agentiloop" cannot be opened* / *unidentified developer* | Esto pasa si lo descargaste con un navegador en lugar de con `curl`. Ejecuta `xattr -d com.apple.quarantine ~/.local/bin/agentiloop` |
| Windows: *Windows protected your PC* | Haz clic en **More info** → **Run anyway** |
| `401` / `invalid x-api-key` / error de autenticación | La clave es incorrecta o tiene espacios o comillas. Cópiala de nuevo y revisa la línea en tu perfil |
| Ollama: modelo no encontrado | Ejecuta `ollama list` y pasa el nombre exacto con `-m` |
| Sigue usando un modelo o proveedor antiguo | Recuerda tus últimas elecciones. Pasa `-p` / `-m` para cambiarlas, o borra `~/.agentiloop/settings.json` para restablecerlo |

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

AgentiLoop necesita un modelo con el que hablar. Elige uno de estos:

| Quiero usar… | Haz esto |
|---|---|
| **Claude** (Anthropic) | `export ANTHROPIC_API_KEY=sk-ant-...` |
| **OpenAI** | `export OPENAI_API_KEY=sk-...` |
| **Ollama, LM Studio** o cualquier servidor compatible con OpenAI | `export OPENAI_BASE_URL=http://localhost:11434/v1` (usa la dirección de tu servidor; los servidores locales no necesitan clave) |
| **oMLX** (modelos locales en Apple Silicon) | Normalmente nada. Inicia oMLX y luego ejecuta AgentiLoop con `-p omlx` (mira más abajo) |

Para Claude puedes usar una clave de API normal (`sk-ant-api…`) o un token de Claude Code (`sk-ant-oat01-…`, que obtienes con `claude setup-token`). AgentiLoop detecta de qué tipo es.

**Detalles de oMLX.** Cuando oMLX se ejecuta en el mismo Mac, AgentiLoop lee el puerto del servidor y la clave de API del propio archivo de configuración de oMLX (`~/.omlx/settings.json`), así que no tienes que exportar nada. Si oMLX se ejecuta en otra máquina, o quieres sobrescribir esa configuración, expórtalos tú mismo:

```sh
export OMLX_BASE_URL=http://192.168.1.50:7777/v1   # la dirección del servidor oMLX (o OMLX_PORT=7777 para localhost)
export OMLX_API_KEY=...                            # la clave de API de la configuración de oMLX
```

Si oMLX tiene desactivada la verificación de la clave de API, no hace falta ninguna clave.

Un `export` solo dura en la pestaña de terminal donde lo escribiste. Para que sea permanente, añade la línea a tu perfil de shell (`~/.zshrc` en macOS). En un Mac puedes guardar la clave en el Llavero en lugar de en el archivo:

```sh
# una sola vez: guarda la clave en tu Llavero
security add-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w "sk-ant-..."

# en ~/.zshrc: cárgala en cada terminal nueva
export ANTHROPIC_API_KEY="$(security find-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w 2>/dev/null)"
```

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
- Las claves de API: esas se quedan en tu perfil de shell

Para olvidarlo todo, borra `~/.agentiloop/settings.json`.

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
| `settings.json` | El proveedor, los modelos y las opciones recordados. Bórralo para restablecer |
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
| `agentiloop-tools` | Herramientas integradas: `read_file`, `write_file`, `edit_file`, `list_dir`, `bash` |
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
