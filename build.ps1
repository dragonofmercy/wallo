# Release build: frontend + Rust + installers (MSI and NSIS) in src-tauri/target/release/bundle.
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

if (-not (Test-Path node_modules)) {
    npm ci
    if ($LASTEXITCODE) { exit $LASTEXITCODE }
}

cargo tauri build
if ($LASTEXITCODE) { exit $LASTEXITCODE }

Get-ChildItem src-tauri/target/release/bundle -Recurse -Include *.msi, *.exe | ForEach-Object { $_.FullName }
