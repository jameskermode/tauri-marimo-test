# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tauri v2 desktop app that wraps the [mograder](https://github.com/jameskermode/mograder) student dashboard for distribution to students on managed university Windows machines (no admin rights required).

The app bundles **uv** (~30MB static binary) as a Tauri sidecar. On first launch, students paste a course configuration URL. uv creates a Python environment and runs `mograder student` which serves the dashboard at `http://127.0.0.1:2718` in the WebView.

## Architecture

- **Tauri v2** app with NSIS installer (`currentUser` install mode — no admin)
- **Sidecar:** `uv` binary at `src-tauri/uv-{target-triple}[.exe]`
- **Rust backend** (`src-tauri/src/main.rs`):
  - `launch_dashboard` command: spawns `uv run --with mograder mograder student <course_dir_or_url> --headless --no-token --port 2718`
  - `on_navigation` plugin: keeps dashboard in WebView, opens external links (Moodle login, marimo edit sessions) in system browser
  - JS init script: intercepts `target="_blank"` links for proper external routing
  - Sidecar stdout/stderr forwarded to frontend via Tauri events
- **Frontend** (`src/index.html`):
  - First launch: setup screen with course URL input
  - Returning user: loading screen → polls localhost:2718 → navigates WebView to dashboard
- **Capabilities** (`src-tauri/capabilities/default.json`): shell:allow-execute for the uv sidecar, core:event:default for frontend event listening

## Build & Development

```bash
# Prerequisites: Rust toolchain, Node.js 20+
npm install                  # frontend deps
npm run tauri dev            # dev mode (hot reload frontend + Rust rebuild)
npm run tauri build          # production build (creates NSIS installer on Windows)
```

The uv sidecar binary must be downloaded and placed at `src-tauri/uv-{target-triple}[.exe]` before building (e.g. `src-tauri/uv-aarch64-apple-darwin` on macOS). CI handles this automatically for all platforms.

For development with a local mograder source, set `MOGRADER_SRC` env var (not currently wired up — use `uv run --with /path/to/mograder` manually or publish to PyPI).

## Key Constraints

- Must install and run without admin rights (per-user install to `%LOCALAPPDATA%`)
- Target: Windows 10/11 managed university machines
- Requires internet on first launch (uv downloads Python + packages)
- WebView2 assumed present on Win 10/11 (Tauri can bundle bootstrapper as fallback)
- Port 2718 hardcoded for mograder/marimo server
- Course config stored in user's app data dir (writable): `%LOCALAPPDATA%/com.mograder-tauri.app/course/`
