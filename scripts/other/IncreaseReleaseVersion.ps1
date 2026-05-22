param(
    [ValidateSet("patch", "minor", "major")]
    [string]$Part = "patch",
    [switch]$Commit,
    [switch]$Tag
)

$ErrorActionPreference = "Stop"

$RepositoryRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $RepositoryRoot

$VersionFile = Join-Path $RepositoryRoot "VERSION.txt"
if (-not (Test-Path -LiteralPath $VersionFile)) {
    Set-Content -LiteralPath $VersionFile -Value "0.1.0" -NoNewline
}

$CurrentVersionText = (Get-Content -LiteralPath $VersionFile -Raw).Trim()
if ($CurrentVersionText -notmatch "^(?<major>\d+)\.(?<minor>\d+)\.(?<patch>\d+)$") {
    throw "VERSION.txt must contain a semantic version like 0.1.0."
}

$Major = [int]$Matches.major
$Minor = [int]$Matches.minor
$Patch = [int]$Matches.patch

switch ($Part) {
    "major" {
        $Major += 1
        $Minor = 0
        $Patch = 0
    }
    "minor" {
        $Minor += 1
        $Patch = 0
    }
    "patch" {
        $Patch += 1
    }
}

$NextVersion = "$Major.$Minor.$Patch"
Set-Content -LiteralPath $VersionFile -Value $NextVersion -NoNewline

$CargoTomls = @(
    "rust/crates/main/Cargo.toml",
    "rust/crates/hot_reload/Cargo.toml",
    "rust/crates/mcp_server/Cargo.toml"
)

foreach ($RelativePath in $CargoTomls) {
    $Path = Join-Path $RepositoryRoot $RelativePath
    $Text = Get-Content -LiteralPath $Path -Raw
    $UpdatedText = $Text -replace '(?m)^version = "\d+\.\d+\.\d+"', "version = `"$NextVersion`""
    Set-Content -LiteralPath $Path -Value $UpdatedText -NoNewline
}

cargo check

if ($Commit) {
    git add VERSION.txt rust/crates/main/Cargo.toml rust/crates/hot_reload/Cargo.toml rust/crates/mcp_server/Cargo.toml Cargo.lock
    git commit -m "Release v$NextVersion"
}

if ($Tag) {
    git tag "v$NextVersion"
}

Write-Host "Version increased from $CurrentVersionText to $NextVersion."
Write-Host "Create a GitHub Release for tag v$NextVersion to publish GitHub Pages."
