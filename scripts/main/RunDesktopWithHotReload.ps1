param(
    [switch]$SkipInstall
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Status {
    param([Parameter(Mandatory = $true)][string]$Message)

    Write-Host "$Message..."
}

function Invoke-QuietCommand {
    param(
        [Parameter(Mandatory = $true)][string]$Status,
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string[]]$Arguments
    )

    Write-Status $Status

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
            Get-Content -Path $stdoutLog -ErrorAction SilentlyContinue
            Get-Content -Path $stderrLog -ErrorAction SilentlyContinue
            throw "$Status failed with exit code $($process.ExitCode)."
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

    $output = & $FilePath @Arguments 2>&1
    $exitCode = $LASTEXITCODE

    if ($exitCode -eq 0) {
        return $true
    }

    Write-Warning "$Status failed with exit code $exitCode. Keeping the running app alive with the previous good build."
    $output | ForEach-Object { Write-Host $_ }
    return $false
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
    Invoke-QuietCommand "Installing" "powershell" @(
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
        (Join-Path $PSScriptRoot "Install.ps1")
    )
}

Invoke-QuietCommand "Building" "cargo" @("build", "--quiet", "-p", "nannou-creative-coding")
Invoke-QuietCommand "HotReloading" "cargo" @("build", "--quiet", "-p", "hot_reload")

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
            throw "Desktop runner exited with code $($runnerProcess.ExitCode)."
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
                    1000
                )
            } until ($change.TimedOut)

            Invoke-HotReloadBuild "HotReloading" "cargo" @("build", "--quiet", "-p", "hot_reload") | Out-Null

            do {
                $change = $watcher.WaitForChanged(
                    [System.IO.WatcherChangeTypes]::Changed -bor
                    [System.IO.WatcherChangeTypes]::Created -bor
                    [System.IO.WatcherChangeTypes]::Deleted -bor
                    [System.IO.WatcherChangeTypes]::Renamed,
                    250
                )
            } until ($change.TimedOut)
        }
    }

    $runnerProcess.Refresh()
    Get-Content -Path $stdoutLog -ErrorAction SilentlyContinue
    Get-Content -Path $stderrLog -ErrorAction SilentlyContinue

    if ($null -ne $runnerProcess.ExitCode -and $runnerProcess.ExitCode -ne 0) {
        throw "Desktop runner exited with code $($runnerProcess.ExitCode)."
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
