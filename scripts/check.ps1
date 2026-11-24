$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
& cargo fmt --all --check
if ($LASTEXITCODE -ne 0) { throw 'Formatting check failed.' }
& cargo test --workspace --locked
if ($LASTEXITCODE -ne 0) { throw 'Tests failed.' }
& cargo clippy --workspace --all-targets --locked -- -D warnings
if ($LASTEXITCODE -ne 0) { throw 'Clippy failed.' }
