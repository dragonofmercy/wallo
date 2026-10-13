# Usage: .\release.ps1 0.2.0
# Bumps the version, moves the CHANGELOG "Unreleased" section under it, builds, then commits, tags, pushes main and the tag,
# and publishes the GitHub release with both installers and latest.json (read by the in-app updater).
#Requires -Version 7.3
param([Parameter(Mandatory)][ValidatePattern('^\d+\.\d+\.\d+$')][string]$Version)
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true
Set-Location $PSScriptRoot
$tag = "v$Version"

if (git status --porcelain) { throw "Working tree is not clean" }
if ((git branch --show-current) -ne "main") { throw "Releases are made from main" }
if (git tag -l $tag) { throw "Tag $tag already exists" }

# The Unreleased section becomes the release notes.
$changelogPath = Join-Path $PSScriptRoot "CHANGELOG.md"
$changelog = [IO.File]::ReadAllText($changelogPath)
$m = [regex]::Match($changelog, '(?s)## \[Unreleased\][ \t]*\n(.*?)(?=\n## \[|\z)')
$notes = $m.Groups[1].Value.Trim()
if (-not $notes) { throw "CHANGELOG.md has nothing under [Unreleased]" }
# Written just before releasing; the heading becomes the version (no empty Unreleased section is kept).
$changelog = $changelog.Remove($m.Index, $m.Length).Insert($m.Index, "## [$Version] - $(Get-Date -Format yyyy-MM-dd)`n`n$notes`n")

# Cargo.toml is the only place holding the version (tauri.conf.json falls back to it).
$cargoPath = Join-Path $PSScriptRoot "src-tauri\Cargo.toml"
$cargo = [regex]::new('(?m)^version = ".*"$').Replace([IO.File]::ReadAllText($cargoPath), "version = `"$Version`"", 1)

[IO.File]::WriteAllText($changelogPath, $changelog)
[IO.File]::WriteAllText($cargoPath, $cargo)
try {
    & (Join-Path $PSScriptRoot "build.ps1")
    if ($LASTEXITCODE) { throw "Build failed" }
} catch {
    git checkout -- CHANGELOG.md src-tauri/Cargo.toml src-tauri/Cargo.lock
    throw
}

$bundle = Join-Path $PSScriptRoot "src-tauri\target\release\bundle"
$setupName = "Wallo_${Version}_x64-setup.exe"
$setup = Join-Path $bundle "nsis\$setupName"
$msi = Join-Path $bundle "msi\Wallo_${Version}_x64_en-US.msi"
$latest = [ordered]@{
    version   = $Version
    notes     = $notes
    pub_date  = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = @{
        "windows-x86_64" = @{
            signature = (Get-Content "$setup.sig" -Raw).Trim()
            url       = "https://github.com/dragonofmercy/wallo/releases/download/$tag/$setupName"
        }
    }
}
$latestPath = Join-Path $bundle "latest.json"
[IO.File]::WriteAllText($latestPath, ($latest | ConvertTo-Json -Depth 4))

git add CHANGELOG.md src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -q -m "Release $tag"
git tag -a $tag -m "Wallo $Version"
git push origin main $tag

$notesFile = New-TemporaryFile
[IO.File]::WriteAllText($notesFile, $notes)
gh release create $tag $setup $msi $latestPath --title "Wallo $Version" --notes-file $notesFile
Remove-Item $notesFile
