mod apps;
mod commands;
mod history;
mod intruder;
mod procctl;
mod state;
mod store;
mod uac;
mod watcher;

use state::AppState;
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_autostart::ManagerExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.unminimize();
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![])))
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::session_status,
            commands::touch,
            commands::lock_session,
            commands::setup,
            commands::login,
            commands::recover,
            commands::change_master,
            commands::set_theme,
            commands::save_settings,
            commands::set_autostart,
            commands::list_installed,
            commands::list_running,
            commands::app_from_path,
            commands::add_app,
            commands::set_app_trust,
            commands::change_app_password,
            commands::set_app_enabled,
            commands::remove_app,
            commands::get_history,
            commands::clear_history,
            commands::unlock_attempt,
            commands::unlock_cancel,
        ])
        .setup(|app| {
            let data = store::load();
            let start_with_windows = data.settings.start_with_windows;
            let state = Arc::new(AppState::new(data));
            app.manage(state.clone());

            // Sincroniza el inicio con Windows con lo guardado.
            let mgr = app.autolaunch();
            if start_with_windows {
                let _ = mgr.enable();
            } else {
                let _ = mgr.disable();
            }

            // Bandeja del sistema.
            let abrir = MenuItemBuilder::with_id("abrir", "Abrir Asegurao").build(app)?;
            let bloquear = MenuItemBuilder::with_id("bloquear", "Bloquear sesión ahora").build(app)?;
            let cerrar = MenuItemBuilder::with_id("cerrar", "Cerrar ventana").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&abrir, &bloquear]).separator().items(&[&cerrar]).build()?;

            let _tray = TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Asegurao — protección activa")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "abrir" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.unminimize();
                            let _ = win.set_focus();
                        }
                    }
                    "bloquear" => {
                        if let Some(st) = app.try_state::<Arc<AppState>>() {
                            st.rt.lock().unwrap().session_unlocked = false;
                        }
                        let _ = app.emit("session:lock", ());
                    }
                    "cerrar" => {
                        if let Some(st) = app.try_state::<Arc<AppState>>() {
                            st.rt.lock().unwrap().session_unlocked = false;
                        }
                        let _ = app.emit("session:lock", ());
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            // Cerrar la ventana principal la oculta (la protección sigue).
            if let Some(win) = app.get_webview_window("main") {
                let w = win.clone();
                let handle = app.handle().clone();
                win.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(st) = handle.try_state::<Arc<AppState>>() {
                            st.rt.lock().unwrap().session_unlocked = false;
                        }
                        let _ = handle.emit("session:lock", ());
                        let _ = w.hide();
                    }
                });
            }

            watcher::spawn(app.handle().clone(), state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error al arrancar Asegurao");
}
