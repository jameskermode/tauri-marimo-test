#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{Emitter, Manager, RunEvent};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

struct SidecarChild(Mutex<Option<CommandChild>>);

/// Launch the mograder student dashboard.
/// `course_dir_or_url` is either a local directory path (returning user)
/// or a URL to a mograder.toml (first-time setup).
#[tauri::command]
fn launch_dashboard(
    app: tauri::AppHandle,
    course_dir_or_url: String,
) -> Result<(), String> {
    let cmd = app.shell().sidecar("uv").map_err(|e| e.to_string())?;
    let cmd = cmd.args([
        "run",
        "--with", "mograder",
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
                    // Intercept target="_blank" links and external links so they
                    // trigger navigation (caught by on_navigation → system browser)
                    // instead of window.open (which Tauri silently blocks).
                    document.addEventListener('click', function(e) {
                        var a = e.target.closest('a[href]');
                        if (!a) return;
                        var href = a.getAttribute('href');
                        if (!href || href.startsWith('#') || href.startsWith('javascript:')) return;
                        // External links or target=_blank → navigate so on_navigation fires
                        if (a.target === '_blank' || (href.startsWith('http') && !href.includes('127.0.0.1:2718'))) {
                            e.preventDefault();
                            e.stopPropagation();
                            window.location.href = href;
                        }
                    }, true);
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
        .invoke_handler(tauri::generate_handler![launch_dashboard])
        .setup(|app| {
            // Resolve course directory in app data (writable by user)
            let app_data_dir = app.path().app_data_dir()
                .expect("failed to resolve app data dir");
            let course_dir = app_data_dir.join("course");
            std::fs::create_dir_all(&course_dir)
                .expect("failed to create course directory");

            // Tell the frontend the course dir and whether setup is needed
            let has_config = course_dir.join("mograder.toml").exists();
            let window = app.get_webview_window("main").unwrap();
            let course_dir_str = course_dir.to_string_lossy().to_string();
            // Emit after a short delay to ensure frontend listeners are ready
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = window.emit("app-init", serde_json::json!({
                    "courseDir": course_dir_str,
                    "hasConfig": has_config,
                }));
            });

            Ok(())
        })
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
