// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod client;
mod server;
mod node;
mod election;
mod health;
mod storage;
mod crypto;
mod api;
mod cli;
mod state;
mod config;

use tauri::{Manager, menu::{Menu, MenuItem}, tray::{TrayIconBuilder, TrayIconEvent}};
use state::AppState;

fn is_wsl() -> bool {
    std::fs::read_to_string("/proc/version")
        .map(|s| s.to_lowercase().contains("microsoft") || s.to_lowercase().contains("wsl"))
        .unwrap_or(false)
}

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Check if running in WSL
    if is_wsl() {
        eprintln!("⚠️  WSL detected - GUI not supported");
        eprintln!("EnvMesh requires a display server to run the GUI.");
        eprintln!("\nOptions:");
        eprintln!("1. Use WSLg (Windows 11) or X server (VcXsrv, Xming)");
        eprintln!("2. Run on native Linux/Windows/macOS");
        eprintln!("3. Wait for CLI-only mode (coming soon)");
        std::process::exit(1);
    }

    tauri::Builder::default()
        .setup(|app| {
            // Get app data directory
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");

            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory");

            let db_path = app_data_dir.join("envmesh.db");

            tracing::info!("Database path: {}", db_path.display());

            // Initialize app state
            let state = tauri::async_runtime::block_on(async {
                AppState::new(db_path).await
                    .expect("Failed to initialize app state")
            });

            app.manage(state);

            // Create system tray menu
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let sync = MenuItem::with_id(app, "sync", "Sync Now", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show, &sync, &quit])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "sync" => {
                        tracing::info!("Manual sync triggered");
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click { .. } => {
                        if let Some(app) = tray.app_handle().get_webview_window("main") {
                            let _ = app.show();
                            let _ = app.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::get_env_var,
            api::set_env_var,
            api::delete_env_var,
            api::list_env_vars,
            api::get_peers,
            api::trigger_sync
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
