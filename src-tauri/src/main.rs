#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{Manager, RunEvent};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandChild;

struct SidecarChild(Mutex<Option<CommandChild>>);

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(SidecarChild(Mutex::new(None)))
        .setup(|app| {
            let cmd = app.shell().sidecar("binaries/uv").map_err(|e| {
                eprintln!("Failed to create sidecar command: {e}");
                e
            })?;

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
