# tauri-marimo-test

Proof-of-concept [Tauri v2](https://tauri.app/) desktop app that bundles [marimo](https://marimo.io/) (Python notebook editor) for distribution to students on managed university Windows machines — no admin rights required.

## How it works

1. The app ships with [uv](https://github.com/astral-sh/uv) (~30 MB) as a Tauri sidecar
2. On launch, uv creates a Python environment and runs `marimo edit --headless` with numpy and matplotlib
3. A loading page polls `http://127.0.0.1:2718` until marimo is ready, then redirects the WebView

First launch requires internet (uv downloads Python + packages). Subsequent launches are fast thanks to uv's cache.

## Building

CI builds the Windows NSIS installer automatically on push to `main`. Download the `.exe` from the [latest workflow run](../../actions/workflows/build.yml).

To build manually on a Windows machine:

```bash
npm install
# Download uv into src-tauri/binaries/uv-x86_64-pc-windows-msvc.exe
npx tauri build
```

The installer is written to `src-tauri/target/release/bundle/nsis/`.

## Requirements

- **End users:** Windows 10/11 with WebView2 (pre-installed) and internet on first launch
- **Building:** Rust toolchain, Node.js 20+, Windows (no cross-compilation)
