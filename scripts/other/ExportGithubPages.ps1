param(
    [string]$Version = "",
    [string]$RepositoryName = "nannou-creative-coding",
    [string]$RepositoryOwner = "SamuelAsherRivello",
    [string]$OutputPath = ""
)

$ErrorActionPreference = "Stop"

$RepositoryRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $RepositoryRoot

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

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $RepositoryRoot "target/github-pages/public"
}

$OutputPath = [System.IO.Path]::GetFullPath($OutputPath)
$RootPath = (Resolve-Path $RepositoryRoot).Path
if (-not $OutputPath.StartsWith($RootPath, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to export outside the repository: $OutputPath"
}

$LatestPath = Join-Path $OutputPath "latest"
$ReleasePath = Join-Path $OutputPath "releases/$TagName"
New-Item -ItemType Directory -Force -Path $LatestPath, $ReleasePath | Out-Null

$ReleaseBuildPath = Join-Path $RepositoryRoot "target/run-app-web/site"
& (Join-Path $PSScriptRoot "ExportWebRelease.ps1") -OutputPath "target/run-app-web/site" -Version $TagName -RepositoryName $RepositoryName -RepositoryOwner $RepositoryOwner
if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
    throw "ExportWebRelease.ps1 failed with exit code $LASTEXITCODE."
}

Copy-Item -Path (Join-Path $ReleaseBuildPath "*") -Destination $LatestPath -Recurse -Force
Copy-Item -Path (Join-Path $ReleaseBuildPath "*") -Destination $ReleasePath -Recurse -Force

Set-Content -LiteralPath (Join-Path $OutputPath ".nojekyll") -Value ""
Copy-Item -Path (Join-Path $LatestPath "404.html") -Destination (Join-Path $OutputPath "404.html") -Force
Set-Content -LiteralPath (Join-Path $OutputPath "index.html") -Value "<!doctype html><meta charset=`"utf-8`"><meta http-equiv=`"refresh`" content=`"0; url=latest/`"><a href=`"latest/`">Latest release</a>"

Write-Host "Exported GitHub Pages files to $OutputPath"
