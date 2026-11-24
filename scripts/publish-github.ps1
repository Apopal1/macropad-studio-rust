# Run only from the restored/cloned Git checkout, after `gh auth login`.
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$repository = 'Apopal1/macropad-studio-rust'
foreach ($tool in @('git', 'gh')) { if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) { throw "Install $tool first." } }
& git rev-parse --show-toplevel
if ($LASTEXITCODE -ne 0) { throw 'Restore the included Git bundle before publishing.' }
$changes = & git status --porcelain
if ($changes) { throw 'Commit or resolve local changes first; publishing requires a clean checkout.' }
& gh auth status
if ($LASTEXITCODE -ne 0) { throw 'Authenticate with gh auth login, then rerun. Do not paste a token into chat.' }
$login = & gh api user --jq .login
if ($LASTEXITCODE -ne 0 -or $login -ne 'Apopal1') { throw 'Expected GitHub account Apopal1. No repository was created.' }
$existing = & gh repo view $repository --json nameWithOwner 2>$null
if ($LASTEXITCODE -eq 0) { throw 'The destination already exists. Stop and inspect it; this script never overwrites an existing repository.' }
$remote = & git remote get-url origin 2>$null
if ($LASTEXITCODE -eq 0) { throw 'An origin remote already exists. Inspect it before publishing.' }
& gh repo create $repository --private --description 'Native Rust macropad studio: egui UI, Windows macros, modes, OLED and Home Assistant' --source . --remote origin --push
if ($LASTEXITCODE -ne 0) { throw 'Creation/push did not complete. Inspect GitHub and origin before retrying.' }
$local = & git rev-parse HEAD
$remoteHead = & git ls-remote origin refs/heads/main
if ($LASTEXITCODE -ne 0 -or -not $remoteHead.StartsWith($local)) { throw 'Remote main could not be verified against local HEAD.' }
& gh repo view $repository --json url,isPrivate
