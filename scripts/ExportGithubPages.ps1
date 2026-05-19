param(
    [string]$Version = "",
    [string]$RepositoryName = "nannou-creative-coding",
    [string]$RepositoryOwner = "SamuelAsherRivello",
    [string]$OutputPath = ""
)

$ErrorActionPreference = "Stop"

$RepositoryRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
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

if (Test-Path -LiteralPath $OutputPath) {
    Remove-Item -LiteralPath $OutputPath -Recurse -Force
}

$LatestPath = Join-Path $OutputPath "latest"
$ReleasePath = Join-Path $OutputPath "releases/$TagName"
New-Item -ItemType Directory -Force -Path $LatestPath, $ReleasePath | Out-Null

$PagesUrl = "https://$($RepositoryOwner.ToLowerInvariant()).github.io/$RepositoryName/"
$RepositoryUrl = "https://github.com/$RepositoryOwner/$RepositoryName"
$ReleaseUrl = "$RepositoryUrl/releases/tag/$TagName"
$GeneratedAt = (Get-Date).ToUniversalTime().ToString("yyyy-MM-dd HH:mm:ss 'UTC'")

function New-ReleasePage {
    param(
        [string]$Title,
        [string]$CanonicalPath
    )

    return @"
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>$Title</title>
  <style>
    :root { color-scheme: light dark; font-family: Inter, Segoe UI, sans-serif; }
    body { margin: 0; min-height: 100vh; display: grid; place-items: center; background: #f7f8f6; color: #151515; }
    main { width: min(720px, calc(100% - 32px)); }
    h1 { font-size: clamp(2rem, 6vw, 4rem); line-height: 1; margin: 0 0 16px; }
    p { font-size: 1.05rem; line-height: 1.6; max-width: 58ch; }
    a { color: #005f73; font-weight: 700; }
    .meta { margin-top: 32px; font-size: .9rem; color: #555; }
    @media (prefers-color-scheme: dark) {
      body { background: #111411; color: #f4f5f0; }
      a { color: #94d2bd; }
      .meta { color: #b7b9b2; }
    }
  </style>
</head>
<body>
  <main>
    <h1>nannou-creative-coding $TagName</h1>
    <p>This is the GitHub Pages export for release <strong>$DisplayVersion</strong>. Download desktop release assets from GitHub Releases.</p>
    <p><a href="$ReleaseUrl">Open the GitHub Release</a></p>
    <p><a href="$RepositoryUrl">View the repository</a></p>
    <p class="meta">Generated $GeneratedAt. Canonical path: $CanonicalPath</p>
  </main>
</body>
</html>
"@
}

$LatestHtml = New-ReleasePage -Title "nannou-creative-coding latest" -CanonicalPath "/latest/"
$ReleaseHtml = New-ReleasePage -Title "nannou-creative-coding $TagName" -CanonicalPath "/releases/$TagName/"

Set-Content -LiteralPath (Join-Path $LatestPath "index.html") -Value $LatestHtml
Set-Content -LiteralPath (Join-Path $ReleasePath "index.html") -Value $ReleaseHtml
Set-Content -LiteralPath (Join-Path $OutputPath ".nojekyll") -Value ""
Set-Content -LiteralPath (Join-Path $OutputPath "404.html") -Value $LatestHtml
Set-Content -LiteralPath (Join-Path $OutputPath "index.html") -Value "<!doctype html><meta charset=`"utf-8`"><meta http-equiv=`"refresh`" content=`"0; url=latest/`"><a href=`"latest/`">Latest release</a>"

Write-Host "Exported GitHub Pages files to $OutputPath"
Write-Host "$PagesUrl"
