#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{Manager, RunEvent};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandChild;

struct SidecarChild(Mutex<Option<CommandChild>>);

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .manage(SidecarChild(Mutex::new(None)))
        .setup(|app| {
            let resource_dir = app.path().resource_dir()
                .expect("failed to resolve resource dir");
            let notebook = resource_dir.join("notebooks").join("test.py");

            let cmd = app.shell().sidecar("uv").map_err(|e| {
                eprintln!("Failed to create sidecar command: {e}");
                e
            })?;

            let notebook_str = notebook.to_string_lossy().to_string();
            let cmd = cmd.args([
                "run",
                "--with", "marimo",
                "--with", "numpy",
                "--with", "matplotlib",
                "marimo", "edit",
                "--headless",
                "--host", "127.0.0.1",
                "--port", "2718",
                "--no-token",
                &notebook_str,
            ]);

            let (_rx, child) = cmd.spawn().map_err(|e| {
                eprintln!("Failed to spawn uv sidecar: {e}");
                e
            })?;

            *app.state::<SidecarChild>().0.lock().unwrap() = Some(child);
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
