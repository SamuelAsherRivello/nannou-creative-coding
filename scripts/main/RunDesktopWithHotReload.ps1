param(
    [switch]$SkipInstall
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Status {
    param([Parameter(Mandatory = $true)][string]$Message)

    Write-Host "$Message..."
}

function Format-ElapsedSeconds {
    param([Parameter(Mandatory = $true)][System.Diagnostics.Stopwatch]$Stopwatch)

    [string]::Format(
        [System.Globalization.CultureInfo]::InvariantCulture,
        "{0:0.0}s",
        $Stopwatch.Elapsed.TotalSeconds
    )
}

function Invoke-QuietCommand {
    param(
        [Parameter(Mandatory = $true)][string]$Status,
        [string]$CompleteStatus,
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string[]]$Arguments
    )

    Write-Status $Status
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

    $stdoutLog = Join-Path ([System.IO.Path]::GetTempPath()) "nannou-creative-coding-$([System.Guid]::NewGuid()).out.log"
    $stderrLog = Join-Path ([System.IO.Path]::GetTempPath()) "nannou-creative-coding-$([System.Guid]::NewGuid()).err.log"

    try {
        $process = Start-Process `
            -FilePath $FilePath `
            -ArgumentList $Arguments `
            -WorkingDirectory (Get-Location) `
            -NoNewWindow `
            -PassThru `
            -Wait `
            -RedirectStandardOutput $stdoutLog `
            -RedirectStandardError $stderrLog

        if ($process.ExitCode -ne 0) {
            $stopwatch.Stop()
            Get-Content -Path $stdoutLog -ErrorAction SilentlyContinue
            Get-Content -Path $stderrLog -ErrorAction SilentlyContinue
            throw "$Status failed with exit code $($process.ExitCode) after $(Format-ElapsedSeconds $stopwatch)."
        }

        $stopwatch.Stop()

        if (-not [string]::IsNullOrWhiteSpace($CompleteStatus)) {
            Write-Host "$CompleteStatus ($(Format-ElapsedSeconds $stopwatch))"
        }
    } finally {
        Remove-Item -LiteralPath $stdoutLog -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $stderrLog -Force -ErrorAction SilentlyContinue
    }
}

function Invoke-HotReloadBuild {
    param(
        [Parameter(Mandatory = $true)][string]$Status,
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string[]]$Arguments
    )

    Write-Status $Status
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

    $output = & $FilePath @Arguments 2>&1
    $exitCode = $LASTEXITCODE
    $stopwatch.Stop()
    $elapsed = Format-ElapsedSeconds $stopwatch

    if ($exitCode -eq 0) {
        Write-Host "$Status Complete ($elapsed)"
        return $true
    }

    Write-Host "$Status Failed ($elapsed)"
    Write-Warning "$Status failed with exit code $exitCode. Keeping the running app alive with the previous good build."
    $output | ForEach-Object { Write-Host $_ }
    return $false
}

function Write-RecompileRequiredMessage {
    Write-Host ""
    Write-Host "You must close and recompile the app."
}

function Stop-ProcessTree {
    param([Parameter(Mandatory = $true)][int]$ProcessId)

    $children = Get-CimInstance Win32_Process -Filter "ParentProcessId = $ProcessId" -ErrorAction SilentlyContinue

    foreach ($child in $children) {
        Stop-ProcessTree -ProcessId $child.ProcessId
    }

    Stop-Process -Id $ProcessId -Force -ErrorAction SilentlyContinue
}

$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $projectRoot

if (-not $SkipInstall) {
    Invoke-QuietCommand "Setup" "" "powershell" @(
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
        (Join-Path $PSScriptRoot "Install.ps1")
    )
}

Invoke-QuietCommand "Building" "Building Complete" "cargo" @("build", "--quiet", "-p", "nannou-creative-coding")
Write-Host ""
Invoke-QuietCommand "HotReloading" "HotReloading Complete" "cargo" @("rustc", "--quiet", "-p", "hot_reload", "--lib", "--crate-type", "dylib")

$stdoutLog = Join-Path ([System.IO.Path]::GetTempPath()) "nannou-creative-coding-runner-$([System.Guid]::NewGuid()).out.log"
$stderrLog = Join-Path ([System.IO.Path]::GetTempPath()) "nannou-creative-coding-runner-$([System.Guid]::NewGuid()).err.log"
$runnerProcess = $null
$watcher = $null

try {
    $runnerProcess = Start-Process `
        -FilePath "cargo" `
        -ArgumentList @("run", "--quiet", "-p", "nannou-creative-coding") `
        -WorkingDirectory $projectRoot `
        -NoNewWindow `
        -PassThru `
        -RedirectStandardOutput $stdoutLog `
        -RedirectStandardError $stderrLog

    Start-Sleep -Milliseconds 500

    if ($runnerProcess.HasExited) {
        $runnerProcess.Refresh()
        Get-Content -Path $stdoutLog -ErrorAction SilentlyContinue
        Get-Content -Path $stderrLog -ErrorAction SilentlyContinue

        if ($null -ne $runnerProcess.ExitCode -and $runnerProcess.ExitCode -ne 0) {
            Write-RecompileRequiredMessage
        }

        return
    }

    $watcher = New-Object System.IO.FileSystemWatcher
    $watcher.Path = Join-Path $projectRoot "rust\crates\hot_reload"
    $watcher.IncludeSubdirectories = $true
    $watcher.EnableRaisingEvents = $true

    while (-not $runnerProcess.HasExited) {
        $change = $watcher.WaitForChanged(
            [System.IO.WatcherChangeTypes]::Changed -bor
            [System.IO.WatcherChangeTypes]::Created -bor
            [System.IO.WatcherChangeTypes]::Deleted -bor
            [System.IO.WatcherChangeTypes]::Renamed,
            500
        )

        if (-not $change.TimedOut) {
            do {
                $change = $watcher.WaitForChanged(
                    [System.IO.WatcherChangeTypes]::Changed -bor
                    [System.IO.WatcherChangeTypes]::Created -bor
                    [System.IO.WatcherChangeTypes]::Deleted -bor
                    [System.IO.WatcherChangeTypes]::Renamed,
                    100
                )
            } until ($change.TimedOut)

            Invoke-HotReloadBuild "HotReloading" "cargo" @("rustc", "--quiet", "-p", "hot_reload", "--lib", "--crate-type", "dylib") | Out-Null

            do {
                $change = $watcher.WaitForChanged(
                    [System.IO.WatcherChangeTypes]::Changed -bor
                    [System.IO.WatcherChangeTypes]::Created -bor
                    [System.IO.WatcherChangeTypes]::Deleted -bor
                    [System.IO.WatcherChangeTypes]::Renamed,
                    100
                )
            } until ($change.TimedOut)
        }
    }

    $runnerProcess.Refresh()
    Get-Content -Path $stdoutLog -ErrorAction SilentlyContinue
    Get-Content -Path $stderrLog -ErrorAction SilentlyContinue

    if ($null -ne $runnerProcess.ExitCode -and $runnerProcess.ExitCode -ne 0) {
        Write-RecompileRequiredMessage
        return
    }
} finally {
    if ($watcher) {
        $watcher.Dispose()
    }

    if ($runnerProcess -and -not $runnerProcess.HasExited) {
        Stop-ProcessTree -ProcessId $runnerProcess.Id
    }

    Remove-Item -LiteralPath $stdoutLog -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $stderrLog -Force -ErrorAction SilentlyContinue
}
