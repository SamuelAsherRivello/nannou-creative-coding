param(
    [string]$OutputPath = "target/run-app-web/site",
    [string]$Version = "",
    [string]$RepositoryName = "nannou-creative-coding",
    [string]$RepositoryOwner = "SamuelAsherRivello"
)

$ErrorActionPreference = "Stop"

$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$ResolvedOutputPath = [System.IO.Path]::GetFullPath((Join-Path $RepositoryRoot $OutputPath))

if (-not $ResolvedOutputPath.StartsWith($RepositoryRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to export outside the repository: $ResolvedOutputPath"
}

if ([string]::IsNullOrWhiteSpace($Version)) {
    $Version = (Get-Content -LiteralPath (Join-Path $RepositoryRoot "VERSION.txt") -Raw).Trim()
}

$Version = $Version.Trim()
if ($Version.StartsWith("v")) {
    $TagName = $Version
    $DisplayVersion = $Version.Substring(1)
} else {
    $TagName = "v$Version"
    $DisplayVersion = $Version
}

& (Join-Path $PSScriptRoot "RunWeb.ps1") -BuildOnly
if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
    throw "RunWeb.ps1 failed with exit code $LASTEXITCODE."
}

if ($ResolvedOutputPath -ne [System.IO.Path]::GetFullPath((Join-Path $RepositoryRoot "target/run-app-web/site"))) {
    New-Item -ItemType Directory -Force -Path $ResolvedOutputPath | Out-Null
    Copy-Item -Path (Join-Path $RepositoryRoot "target/run-app-web/site/*") -Destination $ResolvedOutputPath -Recurse -Force
}

$IndexPath = Join-Path $ResolvedOutputPath "index.html"
$FallbackPath = Join-Path $ResolvedOutputPath "404.html"

if (-not (Test-Path $IndexPath -PathType Leaf)) {
    throw "Expected release index.html was not found: $IndexPath"
}

$IndexContent = Get-Content -Raw -Path $IndexPath
if (-not $IndexContent.Contains("nannou-creative-coding")) {
    throw "Expected release index.html to contain the application title."
}

Copy-Item -Path $IndexPath -Destination $FallbackPath -Force

Write-Host "Exported release web app: $ResolvedOutputPath"
if ($env:GITHUB_OUTPUT) {
    "web_release_output=$OutputPath" | Add-Content -Path $env:GITHUB_OUTPUT -Encoding UTF8
}
