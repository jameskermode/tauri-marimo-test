# Tauri + Marimo: Bundled Windows Install Test

## Goal

Build a minimal Tauri app that bundles marimo as a sidecar, installs to user space on Windows (no admin rights), and opens a marimo notebook in a WebView. This is a proof-of-concept to test whether Tauri can deliver marimo to students on managed university Windows machines.

## Why Tauri

The Marimo team suggested Tauri as a way to distribute marimo as a standalone desktop app. Tauri v2 uses Edge WebView2 (included with Windows 10/11), produces small installers, and defaults to per-user installation in `%LOCALAPPDATA%` — no admin rights required.

## Key Questions to Answer

1. **Does it install on a managed Warwick Windows machine?** (no admin, `%LOCALAPPDATA%` writable?)
2. **Does the .exe run without being blocked by policy?** (SmartScreen, execution policy)
3. **Does WebView2 work?** (should be present on Win 10/11)
4. **Can marimo + Python + JAX all run from the bundled sidecar?**

## Architecture

```
tauri-marimo-app/
├── src-tauri/
│   ├── tauri.conf.json          # Tauri config (NSIS, currentUser, sidecar)
│   ├── Cargo.toml               # Rust dependencies
│   ├── src/
│   │   └── main.rs              # Launch sidecar, open WebView to localhost
│   ├── binaries/                # PyInstaller-bundled marimo sidecar
│   │   └── marimo-server-x86_64-pc-windows-msvc.exe
│   └── capabilities/
│       └── default.json         # Shell permissions for sidecar
├── src/                         # Minimal frontend (just redirects to marimo)
│   └── index.html
└── notebooks/                   # Test notebook(s) bundled as resources
    └── test.py
```

## Step-by-Step Build Instructions

### 1. Create the Python sidecar with PyInstaller

Bundle marimo into a single .exe using PyInstaller. This avoids requiring Python to be installed on the target machine.

```bash
# In a clean venv with marimo and its dependencies
pip install marimo pyinstaller

# Create a wrapper script: marimo_server.py
cat > marimo_server.py << 'EOF'
"""Wrapper to launch marimo server from PyInstaller bundle."""
import sys
import marimo._cli.cli as cli

if __name__ == "__main__":
    sys.argv = ["marimo", "edit", "--headless", "--host", "127.0.0.1", "--port", "2718"]
    # Append notebook path if provided as arg
    if len(sys.argv_original := sys.argv[1:]) > 0:
        sys.argv.extend(sys.argv_original)
    cli.main()
EOF

# Bundle with PyInstaller (single file, console hidden)
pyinstaller --onefile --name marimo-server marimo_server.py
```

The output `dist/marimo-server.exe` is your sidecar binary.

**Important:** Rename to include the target triple:
```bash
mv dist/marimo-server.exe binaries/marimo-server-x86_64-pc-windows-msvc.exe
```

**Note:** If JAX is needed, it must be included in the PyInstaller bundle. This will make the sidecar large (~500MB+). For the initial test, skip JAX and just verify marimo launches.

### 2. Set up the Tauri project

```bash
# Prerequisites: Rust, Node.js
npm create tauri-app@latest -- --template vanilla
cd tauri-marimo-app
```

### 3. Configure tauri.conf.json

Key settings:

```json
{
  "productName": "Marimo Notebooks",
  "version": "0.1.0",
  "identifier": "uk.ac.warwick.sciml.marimo",
  "build": {
    "frontendDist": "../src"
  },
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "externalBin": ["binaries/marimo-server"],
    "resources": ["notebooks/*"],
    "windows": {
      "nsis": {
        "installMode": "currentUser"
      }
    }
  },
  "app": {
    "windows": [
      {
        "title": "Marimo Notebooks",
        "width": 1280,
        "height": 900,
        "url": "http://127.0.0.1:2718"
      }
    ]
  },
  "plugins": {
    "shell": {
      "open": true
    }
  }
}
```

### 4. Configure sidecar permissions

Create `src-tauri/capabilities/default.json`:

```json
{
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    {
      "identifier": "shell:allow-execute",
      "allow": [
        {
          "name": "binaries/marimo-server",
          "sidecar": true
        }
      ]
    }
  ]
}
```

### 5. Rust main.rs — launch sidecar then open WebView

```rust
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Launch marimo server as sidecar
            let sidecar_command = app.shell().sidecar("binaries/marimo-server")
                .expect("failed to create sidecar command");

            let (_rx, _child) = sidecar_command
                .spawn()
                .expect("failed to spawn marimo server");

            // Give marimo a moment to start, then the WebView
            // will connect to http://127.0.0.1:2718
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 6. Minimal frontend (src/index.html)

The WebView URL points directly to the marimo server, so `index.html` is just a fallback:

```html
<!DOCTYPE html>
<html>
<head><title>Loading Marimo...</title></head>
<body>
  <p>Starting marimo server...</p>
  <script>
    // Poll until marimo is ready, then redirect
    const check = setInterval(async () => {
      try {
        const r = await fetch('http://127.0.0.1:2718');
        if (r.ok) {
          clearInterval(check);
          window.location.href = 'http://127.0.0.1:2718';
        }
      } catch(e) {}
    }, 500);
  </script>
</body>
</html>
```

### 7. Build the installer via GitHub Actions

Tauri doesn't support cross-compilation from macOS/Linux to Windows — the NSIS bundling step needs a Windows host. Use GitHub Actions with a `windows-latest` runner.

Create `.github/workflows/build.yml`:

```yaml
name: Build Windows Installer

on:
  push:
    branches: [main]
  workflow_dispatch:

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Install frontend dependencies
        run: npm install

      - name: Download uv binary for Windows
        shell: bash
        run: |
          mkdir -p src-tauri/binaries
          curl -L -o uv.zip https://github.com/astral-sh/uv/releases/latest/download/uv-x86_64-pc-windows-msvc.zip
          unzip uv.zip -d src-tauri/binaries/
          mv src-tauri/binaries/uv.exe src-tauri/binaries/uv-x86_64-pc-windows-msvc.exe

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Upload installer artifact
        uses: actions/upload-artifact@v4
        with:
          name: windows-installer
          path: src-tauri/target/release/bundle/nsis/*.exe
```

Download the artifact from the Actions tab and test on a managed Warwick machine. No Windows dev machine needed for building.

## What to Test on a Managed Warwick Machine

1. **Download the .exe** — does the browser / network policy allow it?
2. **Run the installer** — does it install to `%LOCALAPPDATA%\Marimo Notebooks\` without admin prompt?
3. **Launch the app** — does the WebView open? Does marimo server start?
4. **Open a notebook** — can you edit and run cells?
5. **Check Windows Defender / SmartScreen** — does it block the unsigned app? (Code signing would fix this for production, but for testing we need to know if it's a hard block.)

## Risks and Fallbacks

| Risk | Mitigation |
|------|-----------|
| PyInstaller bundle too large with JAX | Test without JAX first; students can `pip install jax` separately |
| SmartScreen blocks unsigned .exe | Code-sign for production; for testing, users can click "Run anyway" |
| `%LOCALAPPDATA%` blocked on managed machines | Fall back to `pip install marimo` in a user-space venv |
| WebView2 missing | Tauri NSIS can bundle WebView2 bootstrapper; or fall back to system browser |
| Antivirus flags PyInstaller bundle | Known issue with PyInstaller; code-signing helps |

## Option 3 (Recommended): Tauri + uv bootstrap

Instead of bundling a huge PyInstaller sidecar, bundle just `uv` (~30MB static binary) as the sidecar. On first launch, uv creates a Python environment and installs marimo + dependencies in user space. Subsequent launches reuse the cached environment.

This is the cleanest approach: small installer, no Python prerequisite, packages installed lazily on first run, and easy to update dependencies later.

### Architecture

```
tauri-marimo-uv/
├── src-tauri/
│   ├── tauri.conf.json
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs              # Launch uv → marimo, open WebView
│   ├── binaries/
│   │   └── uv-x86_64-pc-windows-msvc.exe   # uv static binary (~30MB)
│   └── capabilities/
│       └── default.json
├── src/
│   └── index.html               # Loading screen while marimo starts
└── notebooks/
    └── test.py
```

### Get the uv binary

Download the standalone uv binary for Windows from https://github.com/astral-sh/uv/releases:

```bash
# Download and rename with target triple
curl -L -o src-tauri/binaries/uv-x86_64-pc-windows-msvc.exe \
  https://github.com/astral-sh/uv/releases/latest/download/uv-x86_64-pc-windows-msvc.zip
# (unzip first — the release is a .zip containing uv.exe)
```

### tauri.conf.json

Same as Option 1 but with `uv` as the sidecar instead of `marimo-server`:

```json
{
  "productName": "Marimo Notebooks",
  "version": "0.1.0",
  "identifier": "uk.ac.warwick.sciml.marimo",
  "build": {
    "frontendDist": "../src"
  },
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "externalBin": ["binaries/uv"],
    "resources": ["notebooks/*"],
    "windows": {
      "nsis": {
        "installMode": "currentUser"
      }
    }
  },
  "app": {
    "windows": [
      {
        "title": "Marimo Notebooks",
        "width": 1280,
        "height": 900,
        "url": "http://127.0.0.1:2718"
      }
    ]
  },
  "plugins": {
    "shell": {
      "open": true
    }
  }
}
```

### capabilities/default.json

```json
{
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    {
      "identifier": "shell:allow-execute",
      "allow": [
        {
          "name": "binaries/uv",
          "sidecar": true,
          "args": true
        }
      ]
    }
  ]
}
```

### Rust main.rs

```rust
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Use uv to run marimo directly — it handles Python
            // installation and dependency resolution automatically.
            // On first run: downloads Python + installs marimo (~30s).
            // On subsequent runs: uses cached environment (~2s).
            let sidecar_command = app.shell()
                .sidecar("binaries/uv")
                .expect("failed to create uv sidecar")
                .args([
                    "run",
                    "--with", "marimo",
                    "--with", "numpy",
                    "--with", "matplotlib",
                    // Add more --with flags for other deps as needed.
                    // For full course: --with jax --with equinox etc.
                    "marimo", "edit",
                    "--headless",
                    "--host", "127.0.0.1",
                    "--port", "2718",
                    "--no-token",
                ]);

            let (_rx, _child) = sidecar_command
                .spawn()
                .expect("failed to spawn uv/marimo");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### src/index.html

Same loading page as Option 1 — polls localhost:2718 until marimo is ready:

```html
<!DOCTYPE html>
<html>
<head><title>Loading Marimo...</title></head>
<body>
  <p>Setting up environment (first launch may take a minute)...</p>
  <script>
    const check = setInterval(async () => {
      try {
        const r = await fetch('http://127.0.0.1:2718');
        if (r.ok) {
          clearInterval(check);
          window.location.href = 'http://127.0.0.1:2718';
        }
      } catch(e) {}
    }, 500);
  </script>
</body>
</html>
```

### Advantages over Option 1 (PyInstaller)

| | Option 1 (PyInstaller) | Option 3 (uv bootstrap) |
|-|----------------------|------------------------|
| Installer size | ~500MB+ (with JAX) | ~30MB (just uv) |
| First launch | Instant | ~30s (downloads Python + deps) |
| Subsequent launches | Instant | ~2s (cached) |
| Updating deps | Rebuild entire sidecar | Change `--with` flags |
| Adding JAX later | Rebuild sidecar | Add `--with jax` |
| Requires internet | No (fully bundled) | Yes, on first launch only |
| Requires Python | No (bundled) | No (uv installs it) |

### What to test on managed Warwick machines

Same as the main test list, plus:
1. **Internet access** — can uv reach pypi.org on first launch? (eduroam should be fine; wired campus network may have proxy)
2. **uv cache location** — uv caches to `%LOCALAPPDATA%\uv\` by default. Verify this is writable.
3. **First-launch time** — how long does the initial `uv run --with marimo` take on a typical student machine?

## Simpler Alternative: No Tauri

If Tauri proves too complex for the initial test, the simpler approach is:

```bash
# Students run this (no admin needed)
pip install --user marimo
marimo edit notebook.py
```

This opens marimo in the system browser. No WebView, no installer, no sidecar. The downside is students need Python installed (which they should have from Term 1 setup).

## References

- [Tauri v2 sidecar docs](https://v2.tauri.app/develop/sidecar/)
- [Tauri Windows installer (NSIS)](https://v2.tauri.app/distribute/windows-installer/)
- [Tauri configuration reference](https://v2.tauri.app/reference/config/)
- [Example: Tauri v2 + Python sidecar](https://github.com/dieharders/example-tauri-v2-python-server-sidecar)
