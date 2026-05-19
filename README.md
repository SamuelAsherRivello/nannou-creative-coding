# nannou-creative-coding

A small Rust and nannou starter project for creative-coding sketches.

## Install Rust

New Windows users can install the Rust toolchain with the project script:

```powershell
.\scripts\Install.ps1
```

The script uses official `rustup`, installs common Rust components, and verifies `rustc`, `cargo`, and `rustup`.

## Run the Sketch

Launch the current sketch once:

```powershell
.\scripts\Run.ps1
```

The app opens an `800x600` nannou window centered on the primary monitor. Press `F` to toggle fullscreen. The app remembers fullscreen state, monitor, position, and size in `target/window-state.json`.

## Run With Hot Reload

Start the development runner:

```powershell
.\scripts\RunWithHotReload.ps1
```

For repeated use after Rust is already installed:

```powershell
.\scripts\RunWithHotReload.ps1 -SkipInstall
```

This script follows the `rksm/nannou-hot-reload` shape: keep the runner thin and put editable sketch code in a dynamic library. It uses `runcc` to run two `cargo-watch` commands together: one keeps the app running, and the other rebuilds the `hot_reload` library. The running app uses `hot-lib-reloader` to load updated `#[no_mangle]` functions without restarting the window.

`.\scripts\Install.ps1` installs the required hot-reload tools:

- `cargo-watch` watches files and rebuilds on changes.
- `runcc` runs the app watcher and library watcher concurrently.

## Files To Edit

Edit [rust/crates/hot-reload/lib.rs](rust/crates/hot-reload/lib.rs) for the sketch code that should hot reload during development:

- `view` controls the drawing.
- `update` controls per-frame behavior.
- `Model` controls sketch state.

Only edit [rust/crates/main/main.rs](rust/crates/main/main.rs) when changing how the binary starts the app, creates the window, handles fullscreen, or wires hot-reload callbacks. It should stay small.

The hot-reload script watches:

- `rust/crates/hot-reload/lib.rs`
- `rust/crates/hot-reload/Cargo.toml`
- `rust/crates/main/main.rs`
- `Cargo.toml`
- `Cargo.lock`

## Run The MCP Server

Start the read-only project helper MCP server over stdio:

```powershell
.\scripts\RunMcpServer.ps1
```

The server is built with the official Rust MCP SDK (`rmcp`) and exposes project helpers for agents:

- `describe_project`
- `hot_reload_target`
- `run_commands`
- `read_project_resource`

It does not execute project commands or modify files.

## GitHub Pages Release Export

Live export URL:

```text
https://samuelasherrivello.github.io/nannou-creative-coding/latest/
```

Versioned exports are published under:

```text
https://samuelasherrivello.github.io/nannou-creative-coding/releases/v0.1.0/
```

Increase the project version locally:

```powershell
.\scripts\IncreaseReleaseVersion.ps1 -Part patch
```

Use `-Part minor` or `-Part major` when needed. The script updates `VERSION.txt` and Rust crate versions, then runs `cargo check`.

Create a release commit and tag:

```powershell
.\scripts\IncreaseReleaseVersion.ps1 -Part patch -Commit -Tag
git push
git push origin v0.1.1
```

Publish a GitHub Release for that tag. The `ExportGithubPages` workflow exports a static GitHub Pages site for `/latest/` and `/releases/<tag>/`.

You can test the export locally without publishing:

```powershell
.\scripts\ExportGithubPages.ps1 -Version v0.1.0
```

The local export is written to `target/github-pages/public`.
