<a href="https://fluxionai.world/register?source=github&campaign=aiagent&promo=AIAGENT"><img src="docs/sponsors/fluxion-ai-silver-ad_de.svg" width="900" alt="Fluxion AI, Silver-Sponsor: eine einheitliche API für GPT, Claude und andere führende KI-Modelle. Spare bis zu 70 % gegenüber den offiziellen API-Preisen und erhalte $3 API-Guthaben." /></a>

# AgentiLoopCLI

🌐 [English](README.md) · [Español](README_es.md) · [Français](README_fr.md) · [Deutsch](README_de.md) · [中文 (简体)](README_zh.md) · [Русский](README_ru.md) · [한국어](README_ko.md) · [日本語](README_ja.md)

### 🎉 Wir haben Version v0.0.4 für Mac, Windows und Linux veröffentlicht!

---

**Probier es aus!** Lies das README und schau, wie lange du brauchst, bis AgentiLoop läuft. Wenn du auf Probleme stößt, sag uns Bescheid. Wir freuen uns über dein Feedback.

**Bonus:** Es gibt auch eine Go-Version: **AgentiLoopGo** → https://github.com/AgentiLoop/AgentiLoopGo

Probier beide aus und sag uns, welche es besser macht: **Rust oder Go?** 🦀 vs 🐹

---

### 💖 Unterstütze AgentiLoop

Gefällt dir, was du siehst? Hilf mit, dass AgentiLoop schnell und plattformübergreifend bleibt. Unterstütze uns über **[GitHub Sponsors → AgentiLoop](https://github.com/sponsors/AgentiLoop)**. Stufen und Vorteile findest du im [Sponsoring-Leitfaden](https://github.com/AgentiLoop/Agent/blob/main/docs/SPONSORSHIP.md).

[![Unterstütze AgentiLoop](https://img.shields.io/badge/Sponsor-AgentiLoop-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white)](https://github.com/sponsors/AgentiLoop)

---

AgentiLoop ist ein KI-Coding-Agent, der in deinem Terminal läuft, ganz im Stil von Claude Code. Du beschreibst in normaler Sprache, was du willst. Der Agent liest deine Dateien, bearbeitet Code und führt Befehle aus, um es umzusetzen, und er fragt dich um Erlaubnis, bevor er irgendetwas ändert.

Er ist in Rust geschrieben und läuft auf macOS, Linux und Windows. Er funktioniert mit Claude (Anthropic), OpenAI, lokalen Modellen über Ollama oder LM Studio sowie oMLX auf Apple Silicon.

Erstellt mit AgentiLoop Agent! Das ist unser Baby. Fertige Binärdateien für macOS, Linux und Windows findest du auf der Seite [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases), oder du kompilierst es selbst aus dem Quellcode mit Rust.

<img src="docs/pong.png" width="900" alt="AgentiLoop schreibt, baut und startet ein Pong-Spiel im Atari-Stil in SwiftUI aus einer einzigen Eingabe, mit Live-Diff und laufendem Spielfenster" />

---

## ⚡ Beim ersten Start: Der Einrichtungsassistent erledigt alles

Es gibt nichts von Hand zu konfigurieren. Wenn du `agentiloop` zum ersten Mal ausführst, startet ein eingebauter Einrichtungsassistent von selbst. Er fragt, welchen Anbieter du möchtest (Claude, OpenAI, Ollama / LM Studio oder oMLX), nimmt deinen API-Schlüssel entgegen (verdeckt eingegeben), prüft, ob der Schlüssel funktioniert, lässt dich ein Modell wählen und speichert alles in `~/.agentiloop`. Etwa eine Minute, keine Konfigurationsdateien, keine `export`-Zeilen.

<img src="docs/setup-wizard-tui.png" width="900" alt="Der Einrichtungsassistent in der Vollbild-Oberfläche (TUI): Anbieter, verdeckter API-Schlüssel, Verbindungstest, Modellliste, Speicherort des Schlüssels, dann die erste Eingabe" />

Du kannst ihn jederzeit mit `agentiloop --setup` erneut starten (füge `--tui` für die Vollbild-Version hinzu, oder tipp `/setup` in einer Sitzung ein). `agentiloop --reset` vergisst alles und fängt ganz von vorn an. Noch nie ein Kommandozeilenprogramm installiert? Folge [Neu hier?](#-neu-hier-in-5-minuten-startklar) weiter unten, Schritt für Schritt.

---

## 🚀 Neu hier? In 5 Minuten startklar

Kein Rust, kein Go, kein Kompilieren. Du lädst eine Datei herunter, gibst ihr einen API-Schlüssel und fängst an zu chatten. Geh die Schritte der Reihe nach durch.

### 1. AgentiLoop herunterladen

Finde zuerst heraus, welche Datei du brauchst:

| Dein Computer | Herunterzuladende Datei |
|---|---|
| Mac mit Apple Silicon (M1, M2, M3, M4…) | `agentiloop-macos-arm64.tar.gz` |
| Mac mit Intel-Chip | `agentiloop-macos-x86_64.tar.gz` |
| Linux, 64-Bit-PC | `agentiloop-linux-x86_64.tar.gz` |
| Linux auf ARM (Raspberry Pi 4/5, ARM-Server) | `agentiloop-linux-arm64.tar.gz` |
| Windows 10/11 | `agentiloop-windows-x86_64.zip` |

Nicht sicher? Führ auf Mac oder Linux `uname -m` aus. `arm64` oder `aarch64` bedeutet **arm64**, und `x86_64` bedeutet **x86_64**.

**macOS und Linux.** Öffne das Terminal und füge diese Zeilen ein. Das Beispiel verwendet die Apple-Silicon-Datei; ändere also `macos-arm64` in den ersten drei Zeilen, falls deine anders ist:

```sh
curl -LO https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.4/agentiloop-macos-arm64.tar.gz
tar xzf agentiloop-macos-arm64.tar.gz
mkdir -p ~/.local/bin && mv agentiloop-macos-arm64/agentiloop ~/.local/bin/
```

Damit landet das Programm in `~/.local/bin`, einem Ordner in deinem Home-Verzeichnis. Der Einrichtungsassistent in Schritt 3 bietet dir an, deinem Terminal zu sagen, dass es dort suchen soll.

**Windows.** Öffne **PowerShell** (Startmenü → "PowerShell" eintippen) und füge ein:

```powershell
Invoke-WebRequest https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.4/agentiloop-windows-x86_64.zip -OutFile agentiloop.zip
Expand-Archive agentiloop.zip -DestinationPath $HOME\agentiloop -Force
$p = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$p;$HOME\agentiloop\agentiloop-windows-x86_64", "User")
```

Die letzten beiden Zeilen fügen AgentiLoop zu deinem PATH hinzu. **Schließ PowerShell und öffne ein neues Fenster**, damit die Änderung übernommen wird.

### 2. Einen API-Schlüssel besorgen

AgentiLoop ist der Agent. Das „Gehirn" ist ein KI-Modell, mit dem du ihn verbindest. Wähle **eins** aus:

| Option | Wo du ihn bekommst | Kosten |
|---|---|---|
| **Claude** (empfohlen) | [console.anthropic.com](https://console.anthropic.com/settings/keys) → *Create Key*. Er beginnt mit `sk-ant-` | Bezahlung nach Nutzung |
| **OpenAI** | [platform.openai.com/api-keys](https://platform.openai.com/api-keys). Er beginnt mit `sk-` | Bezahlung nach Nutzung |
| **Ollama** (läuft auf deinem eigenen Computer) | Installiere es von [ollama.com](https://ollama.com) und führ dann `ollama pull qwen2.5-coder` aus | Kostenlos, kein Schlüssel |

Kopier den Schlüssel an einen sicheren Ort. Du fügst ihn im nächsten Schritt ein.

### 3. Den Einrichtungsassistenten ausführen

Du bearbeitest keine Dateien und tippst keine `export`-Befehle. AgentiLoop hat einen eingebauten Einrichtungsassistenten, der ein paar Fragen stellt und deinen Schlüssel für dich speichert.

Weil `~/.local/bin` noch nicht in deinem PATH ist, startest du ihn dieses eine Mal mit dem vollständigen Pfad (unter Windows hat Schritt 1 den PATH schon angepasst, tipp dort einfach `agentiloop`):

```sh
~/.local/bin/agentiloop
```

Beim ersten Mal, wenn noch kein Schlüssel eingerichtet ist, startet der Assistent von selbst. Er dauert etwa eine Minute und fragt fünf Dinge:

1. **Welcher Anbieter** — Claude, OpenAI, ein lokaler OpenAI-kompatibler Server (Ollama, LM Studio, …) oder oMLX. Tipp eine Zahl ein.
2. **Dein API-Schlüssel** — verdeckt eingegeben, auf dem Bildschirm erscheint nichts. Lokale Server brauchen meist keinen; bei oMLX auf demselben Mac liest der Assistent den Schlüssel aus den Einstellungen von oMLX, er fragt also gar nicht erst.
3. **Verbindungstest** — der Assistent spricht sofort mit dem Anbieter. Ist der Schlüssel falsch, sagt er es dir und bietet einen neuen Versuch an; gespeichert wird nichts, bis es funktioniert.
4. **Welches Modell** — wähle eins aus der Liste, die der Anbieter zurückgibt, oder drück Enter für den Standard. Du kannst es später jederzeit mit `/model` ändern.
5. **Wo der Schlüssel aufbewahrt wird** — drück Enter für den Standard, `~/.agentiloop/env`, eine private Datei, die nur AgentiLoop liest. (Die anderen Optionen, für alle, die den Schlüssel auch in ihrer Shell oder im macOS-Schlüsselbund haben wollen, findest du unter [Erweitert: den Schlüssel von Hand setzen](#schritt-2-ein-modell-verbinden) im Schnellstart.)

Zum Schluss bemerkt der Assistent, dass `~/.local/bin` nicht in deinem PATH ist, und bietet an, es hinzuzufügen. Drück **Enter** (ja). Dann meldet er `All set` und bringt dich zur Eingabezeile. So sieht ein kompletter Durchlauf im normalen Terminal aus, wenn du Claude wählst und die Standardwerte übernimmst (deine Modellliste wird anders aussehen):

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

Du kannst gleich hier mit dem Chatten anfangen, oder du tippst `/exit` und machst mit Schritt 4 weiter. Ab jetzt startet ein einfaches `agentiloop` direkt in der Eingabezeile.

Für Claude kannst du entweder einen normalen API-Schlüssel (`sk-ant-api…`) oder ein Claude-Code-Token (`sk-ant-oat01-…`, aus `claude setup-token`) einfügen; AgentiLoop erkennt, um welche Art es sich handelt. Wenn du Option 3 (Ollama, LM Studio, …) gewählt hast, fragt der Assistent außerdem nach der Server-URL und bietet `http://localhost:11434/v1` als Standard an, für ein lokales Ollama drückst du also einfach Enter.

**Derselbe Assistent, im Vollbild.** Der Assistent läuft in der Oberfläche, die du benutzt. Starte ihn mit `--tui`, und dieselben Fragen erscheinen in der Vollbild-Oberfläche; wenn er fertig ist, bist du schon an der Eingabezeile:

<img src="docs/setup-wizard-tui.png" width="900" alt="Der Einrichtungsassistent in der Vollbild-Oberfläche (TUI): Anbieter, verdeckter API-Schlüssel, Verbindungstest, Modellliste, Speicherort des Schlüssels, dann die erste Eingabe" />

**Jederzeit erneut starten oder neu machen:**

```bash
agentiloop --setup          # Assistent im normalen Terminal
agentiloop --setup --tui    # Assistent in der Vollbild-TUI (wie im Screenshot)
/setup                      # aus einer laufenden Sitzung heraus (REPL oder TUI)
agentiloop --reset          # alles vergessen und ganz von vorn anfangen
```

> 🔒 **Halte deinen Schlüssel geheim.** `~/.agentiloop/env` kann nur von dir gelesen werden. Füge den Schlüssel nicht in Chats ein und committe ihn nicht in git. Auf dem Mac speichert ihn Option 3 bei der Frage „Where should the credential be saved?“ stattdessen im Schlüsselbund, sodass er nie im Klartext auf der Festplatte liegt.

Du möchtest den Schlüssel lieber selbst über Umgebungsvariablen verwalten? Das geht auch, ist aber der Weg für Fortgeschrittene; siehe [Erweitert: den Schlüssel von Hand setzen](#schritt-2-ein-modell-verbinden) im Schnellstart.


### 4. Prüfen, ob es funktioniert

**Öffne ein neues Terminalfenster**, damit es die PATH-Änderung des Assistenten übernimmt (Windows: ein neues PowerShell-Fenster). Dann:

```sh
agentiloop --version
```

Du solltest `agentiloop 0.0.4` sehen. Starte es jetzt ohne Optionen:

```sh
agentiloop
```

Es sollte direkt zur Eingabezeile gehen. Startet stattdessen wieder der Assistent, wurde der Schlüssel nicht gespeichert: Geh Schritt 3 noch einmal durch.

### 5. Deine erste Sitzung

Wechsle in einen Projektordner und starte die Vollbild-Oberfläche:

Fang mit einem neuen, leeren Testordner an, dann kannst du gefahrlos ausprobieren. Für ein echtes Projekt wechselst du stattdessen mit `cd` in dessen Ordner (zum Beispiel `cd ~/code/my-app`).

```sh
mkdir -p ~/agentiloop-test
cd ~/agentiloop-test
agentiloop --tui
```

Du nutzt **Ollama**? Der Assistent hat sich den Server und das gewählte Modell bereits gemerkt. Um zu einem anderen Modell zu wechseln, das du heruntergeladen hast, tipp in der Sitzung `/model` ein.

Jetzt tippst du einfach in normaler Sprache ein, was du willst, und drückst **Enter**. Ein paar gute erste Anfragen:

```text
erklär mir, was dieses Projekt macht
liste die Dateien in src auf und sag mir, welche der Einstiegspunkt ist
finde die TODO-Kommentare und fasse sie zusammen
füge dem Kommandozeilen-Parser eine Option --verbose hinzu
führ die Tests aus und behebe alles, was fehlschlägt
erstelle eine README.md für dieses Projekt
```

Bevor der Agent eine Datei ändert oder einen Befehl ausführt, fragt er dich. Drück **y** für ja, **n** für nein, **a**, um dieses Tool für die Sitzung immer zu erlauben, oder **Esc**, um den Schritt zu überspringen. Mit **Ctrl-C** beendest du das Programm. Beim nächsten Mal startet ein einfaches `agentiloop` genauso und macht mit deiner letzten Unterhaltung weiter.

Du willst nur eine Antwort ohne Chat? Übergib die Frage als Argument:

```sh
agentiloop "explain what this project does"
```

### Was kann er? (Tools)

Der Agent arbeitet mit neun eingebauten Tools. Du rufst sie nicht selbst auf. Du beschreibst das Ziel, und der Agent wählt das Tool:

| Tool | Was es macht | Fragt vorher? |
|---|---|---|
| `read_file` | Liest eine Datei (mit Zeilennummern) | Nein |
| `list_dir` | Listet die Dateien in einem Ordner auf | Nein |
| `glob` | Findet Dateien nach Namensmuster (`*.rs`, `src/**/*.go`) | Nein |
| `grep` | Durchsucht Dateien mit einem regulären Ausdruck | Nein |
| `web_fetch` | Lädt eine Webseite oder API-Antwort als reinen Text | **Ja** |
| `todo_write` | Führt die eigene Checkliste des Modells für mehrstufige Aufgaben | Nein |
| `write_file` | Erstellt eine neue Datei oder überschreibt eine | **Ja** |
| `edit_file` | Ändert eine exakte Textstelle in einer Datei | **Ja** |
| `bash` | Führt einen Shell-Befehl aus, z. B. Tests, Builds oder `git` (`sh -c` auf Mac/Linux, `cmd /C` unter Windows) | **Ja** |

Du willst mehr Tools, etwa Websuche, Datenbanken oder GitHub? Füge MCP-Server hinzu; siehe [Tools mit MCP hinzufügen](#tools-mit-mcp-hinzufügen-optional).

### Der Hilfe-Befehl

`agentiloop --help` listet alle Optionen auf:

```text
$ agentiloop --help
AgentiLoop — a cross-platform agentic coding loop for your terminal

Usage: agentiloop [OPTIONS] [PROMPT]...

Arguments:
  [PROMPT]...  One-shot prompt. If omitted, starts an interactive REPL. A lone `-` in it is replaced by
               what is piped on stdin: git diff | agentiloop "review this" -

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

Tippe in einer Sitzung `/help`, um die Chat-Befehle zu sehen (`/model`, `/sessions`, `/resume`, `/clear`, `/compact`, `/mcp`, `/exit`). Die vollständige Referenz findest du unter [Alle Optionen](#alle-optionen) und [Befehle im Chat](#befehle-im-chat).

### Hängst du fest? Schnelle Lösungen

| Du siehst | Lösung |
|---|---|
| `command not found: agentiloop` | `~/.local/bin` ist nicht in deinem PATH. Öffne zuerst ein neues Terminalfenster; hilft das nicht, führ `~/.local/bin/agentiloop --setup` aus und sag ja, wenn er anbietet, es zum PATH hinzuzufügen. Unter Windows öffnest du ein neues PowerShell-Fenster |
| `Error: no provider credentials found` | Es ist kein Schlüssel gespeichert. Führ `agentiloop --setup` aus (Schritt 3) und prüf es dann mit Schritt 4 |
| macOS: *"agentiloop" cannot be opened* / *unidentified developer* | Das passiert, wenn du mit einem Browser statt mit `curl` heruntergeladen hast. Führ `xattr -d com.apple.quarantine ~/.local/bin/agentiloop` aus |
| Windows: *Windows protected your PC* | Klick auf **More info** → **Run anyway** |
| `401` / `invalid x-api-key` / Authentifizierungsfehler | Der Schlüssel ist falsch oder wurde mit Leerzeichen oder Anführungszeichen eingefügt. Kopier ihn erneut und führ `agentiloop --setup` aus, um ihn neu einzugeben |
| Ollama: Modell nicht gefunden | Führ `ollama list` aus und übergib den exakten Namen mit `-m` |
| Er benutzt immer noch ein altes Modell oder einen alten Anbieter | Er merkt sich deine letzte Wahl. Übergib `-p` / `-m`, um sie zu ändern, oder führe `agentiloop --reset` aus, um von vorn zu beginnen |

Kommst du immer noch nicht weiter? [Eröffne ein Issue](https://github.com/AgentiLoop/AgentiLoopCLI/issues) und füge den Befehl und die Fehlermeldung ein. Wir helfen dir.

---

## Schnellstart

Drei Schritte: installieren, ein Modell verbinden, starten.

### Schritt 1: Installieren

**Herunterladen:** Hol dir das Archiv für deine Plattform unter [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases), entpacke es und leg `agentiloop` (`agentiloop.exe` unter Windows) in deinen PATH.

**Oder selbst bauen:** Wenn du Rust noch nicht hast, installiere es über [rustup.rs](https://rustup.rs). Dann:

```sh
git clone https://github.com/AgentiLoop/AgentiLoopCLI.git
cd AgentiLoopCLI
cargo install --path crates/agentiloop-cli
```

Damit wird das Programm gebaut und ein `agentiloop`-Befehl in deinem PATH abgelegt, in `~/.cargo/bin`.

> **Keine Installation?** Alles in diesem README funktioniert auch direkt im Repo-Ordner. Wo immer du `agentiloop <options>` siehst, tippst du stattdessen `cargo run -- <options>`. Alles nach dem `--` geht an AgentiLoop.

### Schritt 2: Ein Modell verbinden

AgentiLoop braucht ein Modell, mit dem es sprechen kann. **Du musst dafür keine Umgebungsvariablen setzen**: Führe einfach `agentiloop` aus, und der eingebaute Einrichtungsassistent stellt ein paar Fragen und speichert alles für dich. Die Schritt-für-Schritt-Anleitung mit einem kompletten Durchlauf findest du oben unter [3. Den Einrichtungsassistenten ausführen](#3-den-einrichtungsassistenten-ausführen).

<details>
<summary><b>Erweitert: den Schlüssel von Hand setzen</b> (überspring das, wenn der Assistent bei dir funktioniert hat)</summary>

Wenn du den Schlüssel lieber selbst verwalten willst oder AgentiLoop in einem Skript oder in CI läuft, wo niemand dem Assistenten antworten kann, setze eine dieser Umgebungsvariablen, und AgentiLoop benutzt sie, ohne zu fragen:

| Ich möchte nutzen… | Das setzen |
|---|---|
| **Claude** (Anthropic) | `export ANTHROPIC_API_KEY=sk-ant-...` |
| **OpenAI** | `export OPENAI_API_KEY=sk-...` |
| **Ollama, LM Studio** oder einen beliebigen OpenAI-kompatiblen Server | `export OPENAI_BASE_URL=http://localhost:11434/v1` (die Adresse deines Servers; lokale Server brauchen keinen Schlüssel) |
| **oMLX** (lokale Modelle auf Apple Silicon) | Normalerweise nichts. Starte oMLX und führ AgentiLoop dann mit `-p omlx` aus |

**Details zu oMLX.** Wenn oMLX auf demselben Mac läuft, liest AgentiLoop den Server-Port und den API-Schlüssel aus der eigenen Einstellungsdatei von oMLX (`~/.omlx/settings.json`). Läuft oMLX auf einem anderen Rechner oder willst du diese Einstellungen überschreiben:

```sh
export OMLX_BASE_URL=http://192.168.1.50:7777/v1   # die Adresse des oMLX-Servers (oder OMLX_PORT=7777 für localhost)
export OMLX_API_KEY=...                            # der API-Schlüssel aus den oMLX-Einstellungen
```

Wenn in oMLX die Prüfung des API-Schlüssels ausgeschaltet ist, brauchst du keinen Schlüssel.

Ein `export` gilt nur für den Terminal-Tab, in dem du ihn eingegeben hast. Damit er dauerhaft gilt, würdest du die Zeile deinem Shell-Profil hinzufügen (`~/.zshrc` auf macOS), und genau das erledigt die Option **„Also add it to ~/.zshrc"** des Assistenten für dich. Ebenso ist die Option **„macOS Keychain"** des Assistenten die automatische Variante hiervon:

```sh
# einmalig: den Schlüssel im Schlüsselbund speichern
security add-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w "sk-ant-..."

# in ~/.zshrc: ihn für jedes neue Terminal laden
export ANTHROPIC_API_KEY="$(security find-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w 2>/dev/null)"
```

</details>

### Schritt 3: Starten

Wechsle in das Projekt, an dem du arbeiten willst, und starte AgentiLoop:

```sh
cd ~/my-project
agentiloop --tui
```

`--tui` öffnet die Vollbild-Oberfläche, die wir empfehlen. Tipp ein, was du willst, z. B. *„finde heraus, wo die Konfigurationsdatei geladen wird, und füge eine Option --verbose hinzu"*, und drück Enter.

Du siehst die Antworten des Agenten, jedes Tool, das er benutzt (🔧), und jedes Ergebnis (✓ oder ✖). Der Kasten unten zeigt, was er gerade tut, zum Beispiel ` ✻ Thinking...  12s `. Bevor er eine Datei schreibt oder einen Befehl ausführt, fragt er dich:

- **y**: ja, dieses Mal
- **n**: nein
- **a**: dieses Tool für den Rest der Sitzung immer erlauben
- **Esc**: diesen Schritt überspringen, aber weitermachen

---

## Drei Arten, es zu benutzen

| Modus | Befehl | Gut für |
|---|---|---|
| **TUI** (Vollbild) | `agentiloop --tui` | Den Alltag: scrollbarer Verlauf, Live-Status, klickbare Links |
| **Chat** (Zeile für Zeile) | `agentiloop` | Einfache Terminals, oder wenn du reinen Text bevorzugst |
| **Einmalig** | `agentiloop "explain this project"` | Eine einzelne Frage: Er antwortet und beendet sich dann. Praktisch in Skripten |
| **Per Pipe** | `git diff \| agentiloop "review this" -` | Text per Pipe übergeben und `-` im Prompt setzen: Es wird durch den Pipe-Inhalt ersetzt. Praktisch für Diffs und Logs |

Tasten in der TUI: **Enter** sendet · **↑ / ↓** blättern durch frühere Anfragen · **PgUp / PgDn** oder das Mausrad scrollen · **Ctrl-U** löscht die Zeile · **Ctrl-C** beendet.

---

## Er merkt sich deine Einstellungen

Du gibst deine Optionen nur einmal ein. AgentiLoop speichert, wie du es gestartet hast, sodass beim nächsten Mal ein einfaches `agentiloop` genauso startet:

```sh
agentiloop -p anthropic --tui    # beim ersten Mal: Anbieter und TUI wählen
agentiloop                       # ab jetzt: gleicher Anbieter, gleiches Modell, TUI und deine letzte Unterhaltung
```

Was er sich merkt:

- **Anbieter** (`-p`) und **TUI an/aus** (`--tui` / `--no-tui`)
- **Modell**: das zuletzt benutzte, getrennt für jeden Anbieter. Wechselst du zu einem Anbieter zurück, kommt auch sein Modell zurück.
- **Limits**: `--max-turns` und `--compact-at`
- **Deine Unterhaltung**: Er macht mit der letzten Unterhaltung im aktuellen Ordner weiter, sofern diese denselben Anbieter benutzt hat. Die früheren Nachrichten werden wieder auf dem Bildschirm angezeigt, sodass du zurückscrollen und sehen kannst, wo du aufgehört hast

Um etwas zu ändern, übergib die neue Option. Sie gilt sofort und wird ab dann gemerkt:

```sh
agentiloop -p omlx        # zu oMLX wechseln (sein zuletzt benutztes Modell kommt auch zurück)
agentiloop -m <model>     # Modell wechseln
agentiloop --no-tui       # zurück zum zeilenweisen Chat
agentiloop --new          # eine neue Unterhaltung beginnen (die alte bleibt gespeichert)
```

Manche Dinge werden absichtlich **nie** gemerkt:

- `--yes`: Das Überspringen der Berechtigungsabfragen muss jedes Mal eine bewusste Entscheidung sein
- `--no-mcp`, `-C` und einmalige Anfragen
- API-Schlüssel: Die liegen in `~/.agentiloop/env` (vom Assistenten geschrieben) oder in deiner Shell-Umgebung, nie in `settings.json`

Um alles zu vergessen, führe `agentiloop --reset` aus.

---

## Alle Optionen

Jede Option lässt sich auch über eine Umgebungsvariable setzen, die in der zweiten Spalte steht. Eine Option, die du eintippst, hat immer Vorrang vor einem gemerkten Wert.

| Option | Umgebungsvariable | Was sie macht |
|---|---|---|
| `-p, --provider <name>` | `AGENTILOOP_PROVIDER` | `anthropic`, `openai` oder `omlx`. Wenn du keinen angibst, nimmt AgentiLoop den letzten oder erkennt ihn anhand deiner Schlüssel (zuerst Anthropic, dann OpenAI, dann oMLX) |
| `-m, --model <id>` | `AGENTILOOP_MODEL` | Welches Modell benutzt wird |
| `--tui` / `--no-tui` | `AGENTILOOP_TUI` | Vollbild-Oberfläche an / aus |
| `--new` | | Beginnt eine neue Unterhaltung, statt weiterzumachen |
| `-c, --continue` | | Macht hier mit der letzten Unterhaltung weiter (ist schon Standard) |
| `-r, --resume <id>` | | Öffnet eine bestimmte Unterhaltung erneut (IDs findest du mit `/sessions`) |
| `-C, --cwd <folder>` | | Arbeitet in einem anderen Ordner als dem, in dem du gerade bist |
| `--yes` | `AGENTILOOP_YES` | Fragt nicht, bevor Tools ausgeführt werden. ⚠️ Nur für vertrauenswürdige, automatisierte Nutzung |
| `--no-mcp` | `AGENTILOOP_NO_MCP` | Startet keine MCP-Server (siehe unten) |
| `--setup` | | Startet den Einrichtungsassistenten erneut (Anbieter, Schlüssel, Modell). Zusammen mit `--tui` läuft er in der Vollbild-Oberfläche |
| `--reset` | | Zurück auf Anfang: löscht `~/.agentiloop`, den agentiloop-Block in deinem Shell-Profil und die vom Assistenten angelegten Schlüsselbund-Einträge (unter Windows: die von ihm gesetzten Benutzer-Umgebungsvariablen). Von Hand geschriebene `export`-Zeilen werden nur auskommentiert, und nur wenn du zustimmst. Mit `--yes` entfallen die Rückfragen |
| `--max-turns <n>` | | Maximale Anzahl Schritte, die der Agent pro Anfrage machen darf (Standard 50) |
| `--compact-at <tokens>` | `AGENTILOOP_COMPACT_AT` | Wann eine lange Unterhaltung zusammengefasst wird (Standard 150000, `0` = nie) |
| `-h` / `-V` | | Hilfe / Version |

Ein paar Beispiele:

```sh
agentiloop -p openai -m gpt-4o-mini "summarize this repo"    # eine Frage mit einem bestimmten Modell
agentiloop -C ../other-repo --tui                            # an einem anderen Projekt arbeiten
agentiloop --yes "run the tests and fix any failures"        # unbeaufsichtigt, ohne Rückfragen
```

**Welches Modell wird benutzt?** Das erste, das zutrifft, gewinnt:

1. `-m` auf der Kommandozeile
2. das Modell der Unterhaltung, die du fortsetzt
3. das letzte Modell, das du mit diesem Anbieter benutzt hast
4. der Standard des Anbieters: `claude-sonnet-5` für Anthropic, `gpt-4o-mini` für OpenAI oder das erste Modell, das oMLX anbietet

---

## Befehle im Chat

Tipp diese an der Eingabeaufforderung ein, in der TUI oder im Chat:

| Befehl | Was er macht |
|---|---|
| `/model` | Zeigt die verfügbaren Modelle. `/model 3` oder `/model <id>` wechselt (und wird gemerkt) |
| `/sessions` | Listet deine gespeicherten Unterhaltungen auf, die neueste zuerst |
| `/resume <n or id>` | Öffnet eine davon erneut |
| `/clear` | Leert die Unterhaltung und beginnt eine neue |
| `/undo` | Macht die Dateiänderungen des Agenten für deinen letzten Prompt rückgängig (Änderungen durch `bash`-Befehle nicht) |
| `/todos` | Zeigt die aktuelle Aufgaben-Checkliste des Modells |
| `/init` | Erstellt eine `AGENTS.md`-Vorlage für das aktuelle Projekt (überschreibt nie) |
| `/export [file]` | Speichert die Unterhaltung als Markdown-Datei (`/export notes.md` oder ein Standardname im Projektordner) |
| `/compact` | Fasst die Unterhaltung jetzt zusammen, um Platz zu schaffen |
| `/usage` | Zeigt die seit dem Start verbrauchten Tokens und wie voll der Kontext ist |
| `/mcp` | Zeigt die verbundenen MCP-Server und ihre Tools |
| `/help` | Listet diese Befehle auf |
| `/exit` | Beenden |

---

## Lange Unterhaltungen

Modelle können nur eine begrenzte Menge auf einmal lesen. Wenn eine Unterhaltung groß wird (standardmäßig, wenn eine Anfrage 150.000 Tokens erreicht), bittet AgentiLoop das Modell, das Bisherige zusammenzufassen, und macht auf Basis der Zusammenfassung weiter. Du siehst dann einen 📦-Hinweis. `/compact` macht das auf Wunsch, und `--compact-at 0` schaltet es ab.

---

## Projektanweisungen

Lege eine Datei `AGENTS.md` (oder `CLAUDE.md`) in dein Projekt, und AgentiLoop liest sie beim Start und befolgt sie: Build-Befehle, Code-Stil, Dinge, die zu vermeiden sind. Gesucht wird im aktuellen Ordner und dann in den übergeordneten Ordnern bis zum Projektstamm (dem Ordner mit `.git`). Eine persönliche `~/.agentiloop/AGENTS.md` gilt für jedes Projekt; die Datei des Projekts kommt danach und hat Vorrang. Dateien werden bei 32 KB abgeschnitten. Eine Zeile `instructions: <Pfad>` zeigt, welche Dateien geladen wurden.

---

## Tools mit MCP hinzufügen (optional)

[MCP](https://modelcontextprotocol.io)-Server geben dem Agenten zusätzliche Tools, etwa Datenbankzugriff, Websuche oder deine eigenen Skripte. Führ sie in einer JSON-Datei auf:

- `~/.agentiloop/mcp.json`: in jedem Projekt verfügbar
- `.mcp.json` in einem Projektordner: nur in diesem Projekt. Kommt ein Name in beiden Dateien vor, gewinnt diese hier.

Das Format ist dasselbe, das auch Claude Code, Claude Desktop und Agent! benutzen, du kannst also vorhandene Konfigurationen kopieren:

```json
{ "mcpServers": {
    "Local":  { "command": "my-mcp-server", "args": ["--flag"], "env": { "API_KEY": "${MY_KEY}" } },
    "Remote": { "url": "https://example.com/mcp", "headers": { "Authorization": "Bearer ${TOKEN}" } }
} }
```

So funktioniert es:

- **Zwei Arten von Servern.** Ein Server mit `command` ist ein lokales Programm, das AgentiLoop für dich startet. Ein Server mit `url` wird über HTTP erreicht. Sowohl neuere „Streamable HTTP"- als auch ältere „SSE"-Server funktionieren; eine URL, die auf `/sse` endet (oder `"transport": "sse"`), wählt den älteren Stil.
- **Tool-Namen.** Jedes Server-Tool erscheint für den Agenten als `mcp_<server>_<tool>`, z. B. `mcp_Local_search`.
- **Geheimnisse.** `${VAR}` (oder `${VAR:-default}`) wird aus deiner Umgebung befüllt, Schlüssel müssen also nicht in der Datei stehen.
- **Berechtigungen.** MCP-Tools fragen wie jedes andere Tool um Erlaubnis, außer der Server markiert ein Tool als schreibgeschützt.
- **Server abschalten.** Füge `"disabled": true` hinzu, um einen Server zu überspringen, oder starte mit `--no-mcp`, um alle zu überspringen.
- **Sicherheit.** Unverschlüsseltes `http://` ist nur für localhost erlaubt; entfernte Server brauchen `https://`.

Tipp `/mcp`, um zu sehen, welche Server verbunden sind, welche Tools sie haben und ob es Fehler gibt.

---

## Wo alles gespeichert wird

Alles liegt in `~/.agentiloop/`. Setz `AGENTILOOP_HOME`, um einen anderen Ordner zu benutzen, z. B. ein separates Testprofil.

| Datei | Was drinsteht |
|---|---|
| `settings.json` | Gemerkter Anbieter, Modelle und Optionen sowie was der Assistent an anderer Stelle geschrieben hat |
| `env` | Dein Schlüssel, vom Assistenten geschrieben (`KEY=value`, Dateimodus 600). Wird beim Start geladen; ein `export` in deiner Shell hat Vorrang |
| `sessions/` | Deine Unterhaltungen, je eine Datei |
| `mcp.json` | Deine MCP-Server |
| `history.txt` | Anfragen, die du eingetippt hast (für ↑ / ↓) |

---

## Weitere Umgebungsvariablen

Die brauchst du selten:

| Variable | Zweck |
|---|---|
| `ANTHROPIC_BASE_URL` | Schickt Anthropic-Anfragen an einen Proxy oder kompatiblen Server |
| `ANTHROPIC_OAUTH_TOKEN` | Alternative zu `ANTHROPIC_API_KEY` für ein Claude-Code-Token |
| `OMLX_BASE_URL`, `OMLX_PORT`, `OMLX_API_KEY` | Adresse und Schlüssel des oMLX-Servers. Sie überschreiben `~/.omlx/settings.json`, das standardmäßig gelesen wird (Port 8000, wenn keins von beiden gesetzt ist) |
| `RUST_LOG=debug` | Zeigt Debug-Logs, einschließlich des Token-Verbrauchs pro Anfrage |

---

## Für Entwickler

### Bauen und testen

```sh
cargo build --release     # → target/release/agentiloop
cargo test --workspace    # läuft offline, keine API-Schlüssel nötig
```

Die Tests greifen nicht aufs Netzwerk zu. Die Agent-Schleife läuft gegen ein geskriptetes Fake-Modell und die Streaming-Parser gegen einen lokalen Testserver. Der MCP-Client wird gegen einen mitgelieferten Beispielserver über alle drei Verbindungsarten (stdio, HTTP, SSE) getestet. Du kannst diesen Server auch selbst starten, um MCP von Hand auszuprobieren:

```sh
cargo run -p agentiloop-mcp --example mcp-example-server -- --http 8791   # oder --sse 8792, oder --stdio
```

### Wie der Code aufgebaut ist

Das Projekt ist in fünf Crates aufgeteilt, und jedes baut auf den vorherigen auf:

| Crate | Was drinsteckt |
|---|---|
| `agentiloop-core` | Das Herzstück: die Agent-Schleife, Nachrichten, die Tool- und Anbieter-Schnittstellen, Berechtigungen, Sitzungen, Zusammenfassungen |
| `agentiloop-provider` | Spricht mit den Modellen: Anthropic, OpenAI-kompatible Server, oMLX |
| `agentiloop-tools` | Eingebaute Tools: `read_file`, `write_file`, `edit_file`, `list_dir`, `glob`, `grep`, `web_fetch`, `todo_write`, `bash` |
| `agentiloop-mcp` | Der MCP-Client, portiert von AgentMCP aus dem Swift-Code von Agent! |
| `agentiloop-cli` | Das Programm `agentiloop`: Optionen, Chat, TUI, Einstellungen |

### Abhängigkeiten

Wir halten die Abhängigkeiten klein: 18 externe Crates, und jedes Crate listet nur die auf, die es wirklich benutzt.

| Crates | Wofür |
|---|---|
| `tokio`, `futures`, `async-trait` | Nebenläufige Ausführung |
| `serde`, `serde_json` | JSON lesen und schreiben |
| `reqwest` | HTTP-Anfragen an Modelle und MCP-Server |
| `anyhow`, `thiserror` | Fehlerbehandlung |
| `tracing`, `tracing-subscriber` | Logging |
| `dirs` | Deinen Home-Ordner finden |
| `clap` | Kommandozeilenoptionen |
| `rustyline` | Der zeilenweise Chat |
| `ratatui`, `unicode-width`, `textwrap` | Die TUI |
| `pulldown-cmark`, `syntect` | Markdown und Syntaxhervorhebung |

---

## Roadmap

- [x] Gestreamte Antworten
- [x] OpenAI-kompatible Anbieter
- [x] Zusammenfassen langer Unterhaltungen
- [x] Gespeicherte Unterhaltungen
- [x] Vollbild-TUI
- [x] MCP-Client
- [ ] Was kommt als Nächstes (What's NeXT)?

## Lizenz

[PolyForm Noncommercial 1.0.0](LICENSE). Du darfst diese Software für persönliche und nichtkommerzielle Zwecke nutzen, ändern und weitergeben. Die kommerzielle Nutzung, einschließlich des Erstellens oder Verkaufens kommerzieller Versionen, ist AgentiLoop vorbehalten. Wende dich für eine kommerzielle Lizenz an AgentiLoop.

---

<a href="https://fluxionai.world/register?source=github&campaign=aiagent&promo=AIAGENT"><img src="docs/sponsors/fluxion-ai-silver-ad_de.svg" width="900" alt="Fluxion AI, Silver-Sponsor: eine einheitliche API für GPT, Claude und andere führende KI-Modelle. Spare bis zu 70 % gegenüber den offiziellen API-Preisen und erhalte $3 API-Guthaben." /></a>

---

**AgentiLoop:** [agentiloop.ai](https://agentiloop.ai/)

Copyright © 2026 AgentiLoop.ai, a Logos InkPen LLC company. All rights reserved.
