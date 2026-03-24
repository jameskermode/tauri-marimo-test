#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{Emitter, Manager, RunEvent};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

struct SidecarChild(Mutex<Option<CommandChild>>);

/// Return the course directory and whether a config already exists.
#[tauri::command]
fn get_app_state(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let course_dir = app_data_dir.join("course");
    std::fs::create_dir_all(&course_dir).map_err(|e| e.to_string())?;
    let config_path = course_dir.join("mograder.toml");
    let has_config = config_path.exists();
    let title = if has_config {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| {
                s.lines().find_map(|line| {
                    let line = line.trim();
                    if line.starts_with("title") {
                        line.split_once('=')
                            .map(|(_, v)| v.trim().trim_matches('"').trim_matches('\'').to_string())
                    } else {
                        None
                    }
                })
            })
    } else {
        None
    };
    Ok(serde_json::json!({
        "courseDir": course_dir.to_string_lossy(),
        "hasConfig": has_config,
        "title": title,
    }))
}

/// Remove course config so the setup screen is shown on next launch.
#[tauri::command]
fn reset_course(app: tauri::AppHandle) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let course_dir = app_data_dir.join("course");
    if course_dir.exists() {
        std::fs::remove_dir_all(&course_dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Launch the mograder student dashboard.
/// `course_dir_or_url` is either a local directory path (returning user)
/// or a URL to a mograder.toml (first-time setup).
#[tauri::command]
fn launch_dashboard(
    app: tauri::AppHandle,
    course_dir_or_url: String,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let uv_cache = app_data_dir.join("uv-cache");

    let cmd = app.shell().sidecar("uv").map_err(|e| e.to_string())?;
    let cmd = cmd
        .env("TAURI", "1")
        .env("UV_CACHE_DIR", uv_cache.to_string_lossy().to_string())
        .args([
        "run",
        "--refresh",
        "--with", "mograder>=0.1.6",
        "mograder", "student",
        &course_dir_or_url,
        "--headless",
        "--no-token",
        "--port", "2718",
    ]);

    let (mut rx, child) = cmd.spawn().map_err(|e| e.to_string())?;

    // Store child for cleanup on exit
    *app.state::<SidecarChild>().0.lock().unwrap() = Some(child);

    // Forward sidecar stdout/stderr to frontend via Tauri events
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let text = String::from_utf8_lossy(&line);
                    let _ = app_handle.emit("sidecar-output", text.as_ref());
                }
                CommandEvent::Stderr(line) => {
                    let text = String::from_utf8_lossy(&line);
                    let _ = app_handle.emit("sidecar-output", text.as_ref());
                }
                CommandEvent::Error(err) => {
                    let _ = app_handle.emit("sidecar-error", &err);
                }
                _ => {}
            }
        }
    });

    Ok(())
}

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri::plugin::Builder::<tauri::Wry, ()>::new("nav-handler")
                .js_init_script(r#"
                    // Override window.open — Tauri silently blocks new-window requests,
                    // so redirect through navigation for on_navigation to handle.
                    window.open = function(url) {
                        if (url) window.location.href = url;
                    };

                    // Rewrite external/target=_blank links so clicks trigger navigation
                    // (caught by on_navigation → system browser) instead of the native
                    // new-window path which WebKit/Tauri silently swallows.
                    function patchLinks(root) {
                        root.querySelectorAll('a[href]').forEach(function(a) {
                            if (a.dataset.tauriPatched) return;
                            var href = a.getAttribute('href');
                            if (!href || href.startsWith('#') || href.startsWith('javascript:')) return;
                            if (a.target === '_blank' || (href.startsWith('http') && !href.includes('127.0.0.1:2718'))) {
                                a.dataset.tauriPatched = '1';
                                a.removeAttribute('target');
                                a.addEventListener('click', function(e) {
                                    e.preventDefault();
                                    e.stopPropagation();
                                    window.location.href = this.href;
                                }, true);
                            }
                        });
                    }

                    // Patch existing links and watch for new ones
                    new MutationObserver(function() { patchLinks(document); })
                        .observe(document.documentElement, { childList: true, subtree: true });
                    document.addEventListener('DOMContentLoaded', function() { patchLinks(document); });
                "#.to_string())
                .on_navigation(|_webview, url| {
                    let host = url.host_str().unwrap_or_default();

                    // Allow internal Tauri URLs (asset protocol, dev server, about:)
                    if host == "tauri.localhost"
                        || url.scheme() == "tauri"
                        || url.scheme() == "about"
                    {
                        return true;
                    }
                    // Allow localhost/127.0.0.1 on the dashboard port (2718) and
                    // the Tauri dev server (any port that isn't a marimo edit session).
                    // Marimo edit sessions are on high ephemeral ports — we redirect those.
                    if host == "127.0.0.1" || host == "localhost" {
                        match url.port() {
                            Some(2718) => return true,  // dashboard
                            None => return true,        // no port = default (80/443)
                            Some(p) if p < 2718 => return true,  // dev server etc.
                            _ => {}  // high ports = likely marimo edit → open externally
                        }
                    }
                    // Everything else → system browser
                    let _ = open::that(url.as_str());
                    false
                })
                .build(),
        )
        .manage(SidecarChild(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![get_app_state, reset_course, launch_dashboard])
        .setup(|_app| Ok(()))
        .build(tauri::generate_context!())
        .expect("error building tauri app");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            if let Some(child) = handle.state::<SidecarChild>().0.lock().unwrap().take() {
                let _ = child.kill();
            }
        }
    });
}
