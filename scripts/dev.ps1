param([switch]$Demo)
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'Install Rust stable from https://rustup.rs/ first.' }
if ($Demo) { & cargo run -p deck-desktop -- --demo } else { & cargo run -p deck-desktop }
if ($LASTEXITCODE -ne 0) { throw 'Native application failed. See terminal output.' }
