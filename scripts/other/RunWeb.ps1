param(
    [switch]$BuildOnly,
    [int]$Port = 0,
    [switch]$NoOpen
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Test-Command {
    param([Parameter(Mandatory = $true)][string]$Name)

    return $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

function Test-TcpPortAvailable {
    param([Parameter(Mandatory = $true)][int]$Port)

    $listener = $null

    try {
        $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, $Port)
        $listener.Start()
        return $true
    } catch {
        return $false
    } finally {
        if ($null -ne $listener) {
            $listener.Stop()
        }
    }
}

function Find-AvailablePort {
    param(
        [int]$StartPort = 8080,
        [int]$EndPort = 8099
    )

    for ($candidate = $StartPort; $candidate -le $EndPort; $candidate += 1) {
        if (Test-TcpPortAvailable -Port $candidate) {
            return $candidate
        }
    }

    throw "No available localhost TCP port was found from $StartPort through $EndPort."
}

function Set-TextFileIfChanged {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Content
    )

    if (Test-Path -LiteralPath $Path -PathType Leaf) {
        $ExistingContent = Get-Content -LiteralPath $Path -Raw
        if ($ExistingContent -eq $Content) {
            return
        }
    }

    Set-Content -LiteralPath $Path -Value $Content -NoNewline
}

function Get-FileSha256 {
    param([Parameter(Mandatory = $true)][string]$Path)

    $Stream = [System.IO.File]::OpenRead((Resolve-Path -LiteralPath $Path))
    try {
        $Sha256 = [System.Security.Cryptography.SHA256]::Create()
        try {
            return [System.BitConverter]::ToString($Sha256.ComputeHash($Stream)).Replace("-", "")
        } finally {
            $Sha256.Dispose()
        }
    } finally {
        $Stream.Dispose()
    }
}

function Copy-FileIfChanged {
    param(
        [Parameter(Mandatory = $true)][string]$Source,
        [Parameter(Mandatory = $true)][string]$Destination
    )

    if (Test-Path -LiteralPath $Destination -PathType Leaf) {
        $SourceHash = Get-FileSha256 -Path $Source
        $DestinationHash = Get-FileSha256 -Path $Destination
        if ($SourceHash -eq $DestinationHash) {
            return
        }
    }

    Copy-Item -Path $Source -Destination $Destination -Force
}

$RepositoryRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $RepositoryRoot

if (Test-Command "rustup") {
    rustup target add wasm32-unknown-unknown
}

if (-not (Test-Command "wasm-bindgen")) {
    cargo install wasm-bindgen-cli --locked
}

$SitePath = Join-Path $RepositoryRoot "target\run-app-web\site"
$PkgPath = Join-Path $SitePath "pkg"
$CargoTargetPath = Join-Path $RepositoryRoot "target\run-app-web\cargo"
New-Item -ItemType Directory -Force -Path $PkgPath | Out-Null
New-Item -ItemType Directory -Force -Path $CargoTargetPath | Out-Null
Get-ChildItem -LiteralPath $PkgPath -File | Remove-Item -Force

$env:CARGO_TARGET_DIR = $CargoTargetPath
cargo build -p nannou-creative-coding --lib --target wasm32-unknown-unknown --release
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE."
}

$WasmPath = Join-Path $CargoTargetPath "wasm32-unknown-unknown\release\nannou_creative_coding.wasm"
if (-not (Test-Path -LiteralPath $WasmPath -PathType Leaf)) {
    throw "Expected WASM output was not found: $WasmPath"
}

wasm-bindgen $WasmPath --out-dir $PkgPath --target web
if ($LASTEXITCODE -ne 0) {
    throw "wasm-bindgen failed with exit code $LASTEXITCODE."
}

$FaviconPath = Join-Path $SitePath "favicon.svg"
$FaviconSvg = @"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
  <rect width="64" height="64" rx="12" fill="#1b1b18"/>
  <circle cx="32" cy="32" r="18" fill="none" stroke="#f5f5f2" stroke-width="6"/>
  <circle cx="32" cy="32" r="5" fill="#f2b84b"/>
</svg>
"@
Set-TextFileIfChanged -Path $FaviconPath -Content $FaviconSvg
Copy-FileIfChanged -Source $FaviconPath -Destination (Join-Path $SitePath "favicon.ico")

$IndexPath = Join-Path $SitePath "index.html"
$IndexHtml = @"
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="icon" href="./favicon.svg" type="image/svg+xml">
  <title>nannou-creative-coding</title>
  <style>
    html, body { margin: 0; width: 100%; height: 100%; overflow: hidden; background: #000; }
    body { display: grid; place-items: center; }
    canvas {
      display: block;
      aspect-ratio: 16 / 10;
      outline: none;
    }
    #runweb-status {
      position: fixed;
      right: 14px;
      bottom: 14px;
      z-index: 10;
      padding: 8px 10px;
      border: 1px solid rgba(27, 27, 24, 0.16);
      border-radius: 6px;
      background: rgba(245, 245, 242, 0.92);
      color: #1b1b18;
      font: 13px/1.3 system-ui, sans-serif;
    }
    #runweb-status[data-state="success"] {
      border-color: rgba(31, 119, 74, 0.35);
      color: #1f774a;
    }
    #runweb-status[data-state="error"] {
      border-color: rgba(158, 45, 36, 0.35);
      color: #9e2d24;
    }
  </style>
</head>
<body>
  <div id="runweb-status" data-state="starting">Starting WebGPU...</div>
  <script type="module">
    import init from "./pkg/nannou_creative_coding.js";

    const startApp = async () => {
      const status = document.getElementById("runweb-status");
      const setStatus = (state, text) => {
        document.body.dataset.runwebState = state;
        window.__RUNWEB_STATUS = state;
        status.dataset.state = state;
        status.textContent = text;
      };

      if (document.readyState === "loading") {
        await new Promise((resolve) => document.addEventListener("DOMContentLoaded", () => resolve(), { once: true }));
      }

      const focusCanvas = () => {
        const canvas = document.querySelector("canvas");
        if (!canvas) return;

        syncCanvasDisplaySize(canvas);
        canvas.setAttribute("tabindex", "0");
        canvas.focus();
      };

      const syncCanvasDisplaySize = (canvas) => {
        const aspect = 16 / 10;
        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;
        const viewportAspect = viewportWidth / viewportHeight;
        const fittedWidth = viewportAspect > aspect ? viewportHeight * aspect : viewportWidth;
        const maxDisplayWidth = canvas.width;
        const maxDisplayHeight = canvas.height;
        const displayWidth = Math.max(
          1,
          Math.floor(Math.min(fittedWidth, maxDisplayWidth, maxDisplayHeight * aspect))
        );
        const displayHeight = Math.max(1, Math.floor(displayWidth / aspect));

        canvas.style.width = displayWidth + "px";
        canvas.style.height = displayHeight + "px";
      };

      const toggleFullscreen = async () => {
        const target = document.documentElement;
        if (document.fullscreenElement) {
          await document.exitFullscreen();
        } else {
          await target.requestFullscreen();
        }
        focusCanvas();
      };

      document.addEventListener("contextmenu", (event) => {
        if (
          event.target instanceof HTMLCanvasElement ||
          event.target === document.body ||
          event.target === document.documentElement
        ) {
          event.preventDefault();
        }
      });
      window.addEventListener("load", focusCanvas);
      window.addEventListener("resize", focusCanvas);
      window.addEventListener("fullscreenchange", focusCanvas);
      window.addEventListener("pointerdown", focusCanvas);
      const handleFullscreenShortcut = (event) => {
        if (event.key.toLowerCase() !== "f") return;
        if (event.altKey || event.ctrlKey || event.metaKey) return;

        event.preventDefault();
        event.stopImmediatePropagation();
        toggleFullscreen().catch((error) => {
          console.error("Fullscreen toggle failed", error);
        });
      };
      window.addEventListener("keydown", handleFullscreenShortcut, { capture: true });

      if (!navigator.gpu) {
        setStatus("error", "WebGPU unavailable");
        return;
      }

      await init();

      await waitForCanvas();
      if (document.querySelector("canvas")) {
        focusCanvas();
        setStatus("success", "Render success");
      } else {
        setStatus("error", "Canvas was not created");
      }
    };

    const waitForCanvas = async () => {
      for (let frame = 0; frame < 120; frame += 1) {
        if (document.querySelector("canvas")) {
          return true;
        }
        await new Promise((resolve) => requestAnimationFrame(() => resolve()));
      }

      return false;
    };

    const finishStartedEventLoop = async () => {
      await waitForCanvas();
      const canvas = document.querySelector("canvas");
      if (canvas) {
        const status = document.getElementById("runweb-status");
        document.body.dataset.runwebState = "success";
        window.__RUNWEB_STATUS = "success";
        status.dataset.state = "success";
        status.textContent = "Render success";
        return true;
      }
      return false;
    };

    startApp().catch(async (error) => {
      if (String(error && (error.message || error)).includes("Using exceptions for control flow")) {
        if (await finishStartedEventLoop()) {
          return;
        }
      }

      if (sessionStorage.getItem("runweb-webgpu-retried") !== "true") {
        sessionStorage.setItem("runweb-webgpu-retried", "true");
        const status = document.getElementById("runweb-status");
        status.dataset.state = "starting";
        status.textContent = "Retrying WebGPU...";
        setTimeout(() => window.location.reload(), 500);
        return;
      }

      const status = document.getElementById("runweb-status");
      document.body.dataset.runwebState = "error";
      window.__RUNWEB_STATUS = "error";
      window.__RUNWEB_ERROR = String(error && (error.message || error));
      status.dataset.state = "error";
      status.textContent = "WebGPU failed";
      console.error("Failed to start nannou app", error);
    });
  </script>
</body>
</html>
"@
Set-TextFileIfChanged -Path $IndexPath -Content $IndexHtml

Copy-FileIfChanged -Source $IndexPath -Destination (Join-Path $SitePath "404.html")

Write-Host "Built web app: $SitePath"

if ($BuildOnly) {
    return
}

$RequestedPort = $Port
if ($Port -eq 0) {
    $Port = Find-AvailablePort
} elseif (-not (Test-TcpPortAvailable -Port $Port)) {
    throw "Port $Port is not available on 127.0.0.1. Run without -Port to choose the first free port from 8080 through 8099."
}

$Url = "http://127.0.0.1:$Port/"
if ($RequestedPort -eq 0 -and $Port -ne 8080) {
    Write-Host "Port 8080 is unavailable, so using $Port instead."
}
Write-Host "Serving $Url"

if (-not $NoOpen) {
    Start-Process $Url
}

python -m http.server $Port --bind 127.0.0.1 --directory $SitePath
