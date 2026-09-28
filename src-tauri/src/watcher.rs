// Vigilante: hilo en segundo plano que detecta apps bloqueadas, las congela y
// abre la ventana de desbloqueo. También aplica el auto-bloqueo de sesión.

use crate::procctl;
use crate::state::{Allow, AppState};
use serde::Serialize;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Serialize, Clone)]
pub struct Prompt {
    pub exe: String,
    pub display: String,
    pub icon: String,
    pub auth_kind: String,
}

pub fn spawn(app: AppHandle, state: Arc<AppState>) {
    thread::spawn(move || loop {
        tick(&app, &state);
        thread::sleep(Duration::from_millis(250));
    });
}

fn tick(app: &AppHandle, state: &Arc<AppState>) {
    // Auto-bloqueo de sesión.
    {
        let mut rt = state.rt.lock().unwrap();
        let autolock = state.data.lock().unwrap().settings.autolock_minutes;
        if rt.session_unlocked && autolock > 0 {
            if rt.last_activity.elapsed() >= Duration::from_secs(autolock as u64 * 60) {
                rt.session_unlocked = false;
                let _ = app.emit("session:lock", ());
            }
        }
        if rt.pending.is_some() {
            return; // ya hay un desbloqueo en curso; el resto sigue congelado
        }
    }

    // Instantánea de las apps activas.
    let apps: Vec<(String, String, String, String, u32, String)> = {
        let data = state.data.lock().unwrap();
        data.apps
            .iter()
            .filter(|a| a.enabled)
            .map(|a| (a.exe.clone(), a.path.clone(), a.display.clone(), a.icon.clone(), a.trust_minutes, a.auth_kind.clone()))
            .collect()
    };

    for (exe, path, display, icon, _trust, auth_kind) in apps {
        let key = exe.to_lowercase();
        let pids = procctl::running_pids(&exe);
        let running = !pids.is_empty();

        // Limpia permisos caducados.
        {
            let mut rt = state.rt.lock().unwrap();
            let expired = match rt.allowed.get(&key) {
                Some(Allow::Until(when)) => Instant::now() >= *when,
                Some(Allow::UntilExit) => !running,
                None => false,
            };
            if expired {
                rt.allowed.remove(&key);
            }
            if rt.allowed.contains_key(&key) {
                continue; // permitida
            }
        }

        if !running {
            continue;
        }

        // App bloqueada en ejecución y no permitida: congelar y pedir contraseña.
        procctl::suspend(&exe);

        // Guarda la ruta real si aún no la teníamos (para relanzar si hiciera falta).
        if path.is_empty() {
            if let Some(p) = procctl::process_path(pids[0]) {
                let mut data = state.data.lock().unwrap();
                if let Some(a) = data.apps.iter_mut().find(|a| a.exe.eq_ignore_ascii_case(&exe)) {
                    if a.path.is_empty() {
                        a.path = p;
                        let _ = crate::store::save(&data);
                    }
                }
            }
        }

        {
            let mut rt = state.rt.lock().unwrap();
            rt.pending = Some(exe.clone());
        }
        state.prompt_fails.lock().unwrap().remove(&key);

        let prompt = Prompt { exe: exe.clone(), display, icon, auth_kind };
        if let Some(win) = app.get_webview_window("unlock") {
            let _ = win.emit("lock:prompt", prompt.clone());
            let _ = win.show();
            let _ = win.set_focus();
            let _ = win.set_always_on_top(true);
        }
        let _ = app.emit("lock:prompt", prompt);
        return; // uno cada vez
    }
}
