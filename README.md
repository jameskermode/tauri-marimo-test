# tauri-marimo-test

Proof-of-concept [Tauri v2](https://tauri.app/) desktop app that bundles [marimo](https://marimo.io/) (Python notebook editor) for distribution to students on managed university machines — no admin rights required on Windows.

## How it works

1. The app ships with [uv](https://github.com/astral-sh/uv) (~30 MB) as a Tauri sidecar
2. On launch, uv creates a Python environment and runs `marimo edit --headless` with numpy and matplotlib
3. A loading page polls `http://127.0.0.1:2718` until marimo is ready, then redirects the WebView

First launch requires internet (uv downloads Python + packages). Subsequent launches are fast thanks to uv's cache.

## Building

CI builds installers for all three platforms automatically on push to `main`. Download artifacts from the [latest workflow run](../../actions/workflows/build.yml):

| Platform | Artifact | Format |
|----------|----------|--------|
| Windows | `windows-installer` | NSIS `.exe` |
| macOS | `macos-installer` | `.dmg` |
| Linux | `linux-installer` | `.AppImage` |

To build manually:

```bash
npm install
# Download the uv binary for your platform into src-tauri/
# e.g. src-tauri/uv-x86_64-pc-windows-msvc.exe, src-tauri/uv-aarch64-apple-darwin
npx tauri build
```

On Linux, you also need system libraries at build time:

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

## Requirements

- **End users:** Internet on first launch (uv downloads Python + packages)
  - **Windows:** Windows 10/11 with WebView2 (pre-installed)
  - **macOS:** macOS 12+ (Apple Silicon). Unsigned — right-click → Open on first launch to bypass Gatekeeper.
  - **Linux:** `chmod +x` the AppImage and run it
- **Building:** Rust toolchain, Node.js 20+
