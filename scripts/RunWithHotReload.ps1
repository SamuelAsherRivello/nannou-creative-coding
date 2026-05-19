param(
    [switch]$SkipInstall
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Step {
    param([Parameter(Mandatory = $true)][string]$Message)

    Write-Host ""
    Write-Host "==> $Message"
}

function Test-Command {
    param([Parameter(Mandatory = $true)][string]$Name)

    return $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $projectRoot

if (-not $SkipInstall) {
    Write-Step "Checking Rust setup"
    & (Join-Path $PSScriptRoot "Install.ps1")
}

if (-not (Test-Command "cargo-watch")) {
    Write-Step "Installing cargo-watch"
    cargo install cargo-watch --locked
}

if (-not (Test-Command "cargo-runcc")) {
    Write-Step "Installing runcc"
    cargo install runcc --locked
}

Write-Step "Starting hot reload runner"
Write-Host "Running the app and rebuilding the hot_reload dylib when rust/crates/hot-reload/lib.rs changes."
Write-Host "Using workspace target directory: $(Join-Path $projectRoot 'target')"
Write-Host "runcc will stop both commands when you press Ctrl+C."
Write-Host ""

cargo runcc `
    "cargo watch --ignore rust/crates/hot-reload --exec `"run -p nannou-creative-coding`"" `
    "cargo watch --watch rust/crates/hot-reload/lib.rs --watch rust/crates/hot-reload/Cargo.toml --exec `"build -p hot_reload`""
