# Windows smoke test for the setup wizard and --reset, run by CI on windows-latest.
# Drives `--setup` with scripted input against a fake OpenAI-compatible /models server,
# checks that the key landed in HKCU\Environment (the `setx` path Windows uses instead
# of a shell profile), then checks that `--reset` removes it again — including a
# hand-set variable, the way the README's `setx` step leaves it.
#
#   pwsh scripts/windows-smoke.ps1 -Binary .\target\release\agentiloop.exe
param([Parameter(Mandatory = $true)][string]$Binary)

$ErrorActionPreference = 'Stop'
$vars = 'ANTHROPIC_API_KEY', 'ANTHROPIC_OAUTH_TOKEN', 'OPENAI_API_KEY', 'OPENAI_BASE_URL', 'OMLX_BASE_URL', 'OMLX_PORT', 'OMLX_API_KEY'
foreach ($v in $vars + 'SHELL' + 'AGENTILOOP_SHELL_PROFILE') { Remove-Item "Env:$v" -ErrorAction SilentlyContinue }
$home_ = Join-Path ([IO.Path]::GetTempPath()) ("agentiloop-smoke-" + [Guid]::NewGuid())
$env:AGENTILOOP_HOME = $home_

function Assert($cond, $msg) { if (-not $cond) { throw "SMOKE FAIL: $msg" } }
function InUserEnv($name) { $null -ne [Environment]::GetEnvironmentVariable($name, 'User') }

# Fake /v1/models server.
$server = @'
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
class H(BaseHTTPRequestHandler):
    def do_GET(self):
        b = json.dumps({"object": "list", "data": [{"id": "alpha"}, {"id": "beta"}]}).encode()
        self.send_response(200); self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(b))); self.end_headers(); self.wfile.write(b)
    def log_message(self, *a): pass
HTTPServer(("127.0.0.1", 18434), H).serve_forever()
'@
$py = Join-Path ([IO.Path]::GetTempPath()) 'agentiloop-fake-models.py'
Set-Content -Path $py -Value $server
$proc = Start-Process python -ArgumentList $py -PassThru -WindowStyle Hidden
try {
    Start-Sleep -Seconds 2

    # --setup: provider 3 (local server), URL, no key, model 2, save option 2 = user env var (setx).
    $answers = "3`nhttp://127.0.0.1:18434/v1`n`n2`n2`n"
    $out = $answers | & $Binary --setup --no-mcp 2>&1 | Out-String
    Write-Host $out
    Assert ($LASTEXITCODE -eq 0) "--setup exited $LASTEXITCODE"
    Assert ($out -match 'All set: openai / beta') "wizard did not finish with openai/beta"
    Assert (Test-Path (Join-Path $home_ 'env')) "env file not written"
    Assert ((Get-Content (Join-Path $home_ 'env') -Raw) -match 'OPENAI_BASE_URL=http://127.0.0.1:18434/v1') "env file lacks OPENAI_BASE_URL"
    Assert ((Get-Content (Join-Path $home_ 'settings.json') -Raw) -match '"user_env":\s*\[\s*"OPENAI_BASE_URL"') "settings.json lacks setup.user_env"
    Assert (InUserEnv 'OPENAI_BASE_URL') "setx did not persist OPENAI_BASE_URL to HKCU\Environment"

    # A hand-set variable (the README's setx path) must be found and, with --yes, removed.
    setx OPENAI_API_KEY 'hand-set' | Out-Null
    Assert (InUserEnv 'OPENAI_API_KEY') "test setup: setx OPENAI_API_KEY failed"

    $out = & $Binary --reset --yes 2>&1 | Out-String
    Write-Host $out
    Assert ($LASTEXITCODE -eq 0) "--reset exited $LASTEXITCODE"
    Assert ($out -match 'remove the Windows user environment variable `OPENAI_BASE_URL`') "reset did not list the wizard's variable"
    Assert ($out -match 'set by hand \(`setx`\).*OPENAI_API_KEY') "reset did not detect the hand-set variable"
    Assert ($out -match 'Remove-Item Env:') "reset hint is not PowerShell-flavoured"
    Assert (-not (Test-Path $home_)) "AGENTILOOP_HOME still exists after --reset"
    Assert (-not (InUserEnv 'OPENAI_BASE_URL')) "OPENAI_BASE_URL still in HKCU\Environment after --reset"
    Assert (-not (InUserEnv 'OPENAI_API_KEY')) "hand-set OPENAI_API_KEY still in HKCU\Environment after --reset --yes"

    $out = & $Binary --reset --yes 2>&1 | Out-String
    Assert ($out -match 'Nothing to reset') "second --reset was not a no-op"
    Write-Host "Windows smoke test passed."
}
finally {
    Stop-Process -Id $proc.Id -ErrorAction SilentlyContinue
    foreach ($v in 'OPENAI_BASE_URL', 'OPENAI_API_KEY') { [Environment]::SetEnvironmentVariable($v, $null, 'User') }
    Remove-Item $home_ -Recurse -Force -ErrorAction SilentlyContinue
}
