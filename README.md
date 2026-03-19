<p align="center">
  <img src="src-tauri/icons/icon.png" alt="mograder" width="128" />
</p>

<h1 align="center">mograder-tauri</h1>

<p align="center">
  <a href="https://tauri.app/">Tauri v2</a> desktop app that wraps the <a href="https://github.com/jameskermode/mograder">mograder</a> student dashboard for distribution to students on managed university machines — no admin rights required on Windows.
</p>

## How it works

1. The app ships with [uv](https://github.com/astral-sh/uv) (~30 MB) as a Tauri sidecar
2. On first launch, a setup screen asks for a course configuration URL (provided by the instructor)
3. uv creates a Python environment and runs `mograder student` which serves the dashboard at `http://127.0.0.1:2718`
4. The WebView displays the dashboard; external links (Moodle login, marimo edit sessions) open in the system browser

Subsequent launches skip setup and go straight to the dashboard. First launch requires internet (uv downloads Python + packages).

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
  - **Windows:** Windows 10/11 with WebView2 (pre-installed). Unsigned — click "Run Anyway" on SmartScreen prompt.
  - **macOS:** macOS 12+ (Apple Silicon). Unsigned — right-click → Open on first launch to bypass Gatekeeper.
  - **Linux:** `chmod +x` the AppImage and run it
- **Building:** Rust toolchain, Node.js 20+

## Testing

On first launch, the app shows a setup screen asking for a course configuration URL. For testing, use the mograder demo course (HTTPS transport, no authentication required):

```
https://raw.githubusercontent.com/jameskermode/mograder/main/demo/codespaces/mograder.toml
```

Paste this URL into the setup screen and click "Connect".
