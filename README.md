<p align="center">
  <img src="src-tauri/icons/icon.png" alt="mograder" width="128" />
</p>

<h1 align="center">mograder</h1>

<p align="center">
  Desktop app for browsing, editing, and submitting course assignments.<br/>
  No admin rights required — just download, install, and go.
</p>

## Download

Go to the [latest build](../../actions/workflows/build.yml), click the most recent green run, and download the installer for your platform:

| Platform | Download |
|----------|----------|
| Windows | `windows-installer` |
| macOS | `macos-installer` |
| Linux | `linux-installer` |

## Install

**Windows** — Run the `.exe` installer. If Windows Defender SmartScreen appears, click **More info** then **Run anyway**. No admin rights needed.

**macOS** — Open the `.dmg`, drag the app to Applications. On first launch, right-click the app and choose **Open** (needed once because the app is unsigned).

**Linux** — Download the `.AppImage`, make it executable (`chmod +x mograder-tauri_*.AppImage`), and run it.

## Getting started

1. **Launch the app.** On first launch, you'll see a setup screen.
2. **Paste the course URL** your instructor gave you, then click **Connect**.
3. **Wait for setup** — the app downloads everything it needs automatically. This takes a minute or two the first time; subsequent launches are fast.
4. **Log in** when the dashboard appears (e.g. with your Moodle token if your course uses Moodle).

Once set up, you can browse assignments, download notebooks, edit them in [marimo](https://marimo.io/), validate your work, and submit — all from the dashboard.

To switch to a different course, click **Change course** in the toolbar.

## Troubleshooting

- **"Waiting for server..." doesn't stop** — Check your internet connection. The first launch needs to download Python and course packages.
- **Moodle login link doesn't work** — Copy the token URL shown in the dashboard and paste it into your browser manually.
- **App won't open on macOS** — Right-click the app and choose Open. You only need to do this once.

## For instructors

See the [mograder documentation](https://github.com/jameskermode/mograder) for how to set up a course and generate the configuration URL to share with students.

---

<details>
<summary>Developer notes</summary>

### Architecture

The app is built with [Tauri v2](https://tauri.app/) and bundles [uv](https://github.com/astral-sh/uv) as a sidecar. On launch, uv creates a Python environment and runs `mograder student` which serves a [marimo](https://marimo.io/)-based dashboard at `http://127.0.0.1:2718`. The dashboard is displayed in an iframe within the app's WebView.

### Building from source

Requires: Rust toolchain, Node.js 20+

```bash
npm install
# Download the uv binary for your platform into src-tauri/
# e.g. src-tauri/uv-x86_64-pc-windows-msvc.exe, src-tauri/uv-aarch64-apple-darwin
npx tauri build
```

On Linux, install system dependencies first:

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

### CI

Installers for Windows, macOS, and Linux are built automatically on push to `main` via GitHub Actions.

</details>
