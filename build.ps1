# Release build: frontend + Rust + installers (MSI and NSIS) in src-tauri/target/release/bundle.
# The updater artifacts (.sig) are signed with ../keys/updater.key, kept outside the repo.
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

$key = Join-Path $PSScriptRoot "..\keys\updater.key"
if (-not (Test-Path $key)) { throw "Updater signing key not found: $key" }
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""

if (-not (Test-Path node_modules)) {
    npm ci
    if ($LASTEXITCODE) { exit $LASTEXITCODE }
}

cargo tauri build
if ($LASTEXITCODE) { exit $LASTEXITCODE }

Get-ChildItem src-tauri/target/release/bundle -Recurse -Include *.msi, *.exe | ForEach-Object { $_.FullName }
