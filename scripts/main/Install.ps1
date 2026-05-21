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

function Add-CargoBinToPath {
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"

    if ((Test-Path $cargoBin) -and ($env:Path -notlike "*$cargoBin*")) {
        $env:Path = "$cargoBin;$env:Path"
    }
}

$isWindowsPlatform = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
    [System.Runtime.InteropServices.OSPlatform]::Windows
)

if (-not $isWindowsPlatform) {
    throw "This installer is intended for Windows PowerShell. Visit https://www.rust-lang.org/tools/install for other platforms."
}

Write-Step "Checking for Rust"

if (Test-Command "rustup") {
    Write-Host "rustup is already installed."
} elseif (-not (Test-Command "cargo")) {
    Write-Step "Installing Rust with rustup"

    $installerPath = Join-Path ([System.IO.Path]::GetTempPath()) "rustup-init.exe"
    $installerUrl = "https://win.rustup.rs/x86_64"

    Write-Host "Downloading $installerUrl"
    Invoke-WebRequest -Uri $installerUrl -OutFile $installerPath

    Write-Host "Running rustup-init.exe"
    & $installerPath -y --default-toolchain stable

    Add-CargoBinToPath
} else {
    Write-Host "cargo is already installed. rustup was not found, so no toolchain update was attempted."
}

Add-CargoBinToPath

if (Test-Command "rustup") {
    Write-Step "Installing common Rust components"
    rustup component add rustfmt clippy

    Write-Step "Installing WASM target"
    rustup target add wasm32-unknown-unknown
}

Write-Step "Verifying installation"
rustc --version
cargo --version

if (Test-Command "rustup") {
    rustup --version
}

Write-Step "Installing hot reload tools"

if (-not (Test-Command "cargo-watch")) {
    cargo install cargo-watch --locked
} else {
    Write-Host "cargo-watch is already installed."
}

if (-not (Test-Command "cargo-runcc")) {
    cargo install runcc --locked
} else {
    Write-Host "runcc is already installed."
}

Write-Step "Installing WASM tools"

if (-not (Test-Command "wasm-bindgen")) {
    cargo install wasm-bindgen-cli --locked
} else {
    Write-Host "wasm-bindgen-cli is already installed."
}

Write-Host ""
Write-Host "Rust is ready. Run .\scripts\other\RunDesktop.ps1 for desktop or .\scripts\other\RunWeb.ps1 for web."
