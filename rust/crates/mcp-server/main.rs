use anyhow::Result;
use rmcp::{
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ServerHandler, ServiceExt,
};
use serde::Deserialize;
use std::{fs, path::PathBuf, process::Command};

#[derive(Clone)]
struct ProjectServer {
    tool_router: ToolRouter<Self>,
}

impl ProjectServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ResourceRequest {
    uri: String,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct ScreenshotRequest {
    output_path: Option<String>,
}

#[tool_router]
impl ProjectServer {
    #[tool(description = "Describe the nannou creative-coding project layout.")]
    fn describe_project(&self) -> String {
        [
            "nannou-creative-coding is a Rust workspace for nannou sketches.",
            "The app runner lives at rust/crates/main/main.rs.",
            "Hot-reloadable drawing code lives at rust/crates/hot-reload/lib.rs.",
            "The hot reload script is scripts/RunWithHotReload.ps1.",
        ]
        .join("\n")
    }

    #[tool(
        description = "Return the file path developers should edit for hot-reloaded drawing changes."
    )]
    fn hot_reload_target(&self) -> String {
        "rust/crates/hot-reload/lib.rs".to_string()
    }

    #[tool(description = "List supported local project commands without executing them.")]
    fn run_commands(&self) -> String {
        [
            ".\\scripts\\Install.ps1",
            ".\\scripts\\Run.ps1",
            ".\\scripts\\RunWithHotReload.ps1",
            "cargo fmt --check",
            "cargo check",
            "cargo build -p hot_reload",
            ".\\scripts\\RunMcpServer.ps1",
            "MCP tool: take_screenshot",
        ]
        .join("\n")
    }

    #[tool(description = "Read a small project resource by URI.")]
    fn read_project_resource(&self, Parameters(request): Parameters<ResourceRequest>) -> String {
        match request.uri.as_str() {
            "nannou://guide" => "https://guide.nannou.cc/".to_string(),
            "project://readme" => include_str!("../../../README.md").to_string(),
            _ => format!(
                "Unknown resource URI: {}. Supported URIs: nannou://guide, project://readme",
                request.uri
            ),
        }
    }

    #[tool(
        description = "Take a DPI-aware screenshot of the full nannou app window by title and write it to target/screenshots."
    )]
    fn take_screenshot(&self, Parameters(request): Parameters<ScreenshotRequest>) -> String {
        let output_path = request
            .output_path
            .map(PathBuf::from)
            .unwrap_or_else(default_screenshot_path);
        let output_path = if output_path.is_absolute() {
            output_path
        } else {
            match std::env::current_dir() {
                Ok(current_dir) => current_dir.join(output_path),
                Err(error) => return format!("Failed to resolve current directory: {error}"),
            }
        };

        if let Some(parent) = output_path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                return format!("Failed to create screenshot directory: {error}");
            }
        }

        let script = screenshot_script(&output_path);
        match Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &script,
            ])
            .output()
        {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if stdout.is_empty() {
                    output_path.display().to_string()
                } else {
                    stdout
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                format!(
                    "Screenshot failed with status {}.\nstdout:\n{}\nstderr:\n{}",
                    output.status, stdout, stderr
                )
            }
            Err(error) => format!("Failed to run PowerShell screenshot capture: {error}"),
        }
    }
}

fn default_screenshot_path() -> PathBuf {
    target_path().join("screenshots").join("nannou-app.png")
}

fn screenshot_script(output_path: &PathBuf) -> String {
    let output_path = ps_single_quoted(&output_path.display().to_string());

    format!(
        r#"
$ErrorActionPreference = "Stop"
$outPath = {output_path}
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class NannouScreenshot {{
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    public struct RECT {{ public int Left; public int Top; public int Right; public int Bottom; }}
}}
'@
[NannouScreenshot]::SetProcessDPIAware() | Out-Null
$proc = Get-Process -Name nannou-creative-coding -ErrorAction Stop | Where-Object {{ $_.MainWindowHandle -ne 0 }} | Select-Object -First 1
if ($null -eq $proc) {{ throw "nannou-creative-coding window was not found." }}
[NannouScreenshot]::SetForegroundWindow($proc.MainWindowHandle) | Out-Null
Start-Sleep -Milliseconds 300
$rect = New-Object NannouScreenshot+RECT
[NannouScreenshot]::GetWindowRect($proc.MainWindowHandle, [ref]$rect) | Out-Null
$width = $rect.Right - $rect.Left
$height = $rect.Bottom - $rect.Top
if ($width -le 0 -or $height -le 0) {{ throw "Invalid window rectangle: $width x $height." }}
$bitmap = New-Object System.Drawing.Bitmap $width, $height
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
$bitmap.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose()
$bitmap.Dispose()
$outPath
"#
    )
}

fn ps_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn target_path() -> PathBuf {
    PathBuf::from("target")
}

#[tool_handler]
impl ServerHandler for ProjectServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Read-only project helper for the nannou creative-coding workspace.".into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let service = ProjectServer::new().serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
