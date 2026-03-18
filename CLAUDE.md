# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Proof-of-concept Tauri v2 desktop app that bundles [marimo](https://marimo.io/) (Python notebook editor) for distribution to students on managed university Windows machines (no admin rights required).

The preferred architecture (Option 3 in the design doc) bundles **uv** (~30MB static binary) as a Tauri sidecar. On first launch, uv creates a Python environment and installs marimo + dependencies. The WebView points to `http://127.0.0.1:2718` where marimo serves its UI.

## Status

**Scaffolded.** The Tauri v2 project structure is in place. To build/run, download the platform-appropriate `uv` binary into `src-tauri/` and run `npx tauri dev`.

## Architecture (uv bootstrap approach)

- **Tauri v2** app with NSIS installer (`currentUser` install mode — no admin)
- **Sidecar:** `uv` binary at `src-tauri/uv-{target-triple}[.exe]`
- **Rust backend** (`src-tauri/src/main.rs`): spawns `uv run --with marimo ... marimo edit --headless --port 2718`
- **Frontend** (`src/index.html`): loading page that polls localhost:2718 until marimo is ready, then redirects
- **Notebooks** bundled as resources in `notebooks/`
- **Capabilities** (`src-tauri/capabilities/default.json`): shell:allow-execute for the uv sidecar

## Build & Development

```bash
# Prerequisites: Rust toolchain, Node.js 20+
npm install                  # frontend deps
npm run tauri dev            # dev mode (hot reload frontend + Rust rebuild)
npm run tauri build          # production build (creates NSIS installer on Windows)
```

The uv sidecar binary must be downloaded and placed at `src-tauri/uv-{target-triple}[.exe]` before building (e.g. `src-tauri/uv-aarch64-apple-darwin` on macOS). CI handles this automatically for all platforms.

## Key Constraints

- Must install and run without admin rights (per-user install to `%LOCALAPPDATA%`)
- Target: Windows 10/11 managed university machines
- Requires internet on first launch (uv downloads Python + packages)
- WebView2 assumed present on Win 10/11 (Tauri can bundle bootstrapper as fallback)
- Port 2718 hardcoded for marimo server
