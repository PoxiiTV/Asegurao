// Vigilante: hilo en segundo plano que detecta apps bloqueadas, las congela y
// abre la ventana de desbloqueo. También aplica el auto-bloqueo de sesión.
//
// Congela TODO lo bloqueado en cuanto aparece, aunque ya haya otra petición de
// contraseña abierta: las peticiones van de una en una y el resto espera
// congelado.

use crate::procctl::{self, Live, Scanner};
use crate::state::{Allow, AppState};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

// Tiempo sin ventana visible para dar por "escondida en la bandeja" una app.
const HIDE_GRACE: Duration = Duration::from_secs(2);

#[derive(Serialize, Clone)]
pub struct Prompt {
    pub exe: String,
    pub display: String,
    pub icon: String,
    pub auth_kind: String,
}

pub fn spawn(app: AppHandle, state: Arc<AppState>) {
    thread::spawn(move || {
        let mut scanner = Scanner::default();
        loop {
            tick(&app, &state, &mut scanner);
            thread::sleep(Duration::from_millis(250));
        }
    });
}

fn tick(app: &AppHandle, state: &Arc<AppState>, scanner: &mut Scanner) {
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
    }

    let live: Vec<Live> = scanner.scan();
    let visible = procctl::visible_window_pids();

    // Instantánea de las apps activas.
    let apps: Vec<(String, String, String, String, String)> = {
        let data = state.data.lock().unwrap();
        data.apps
            .iter()
            .filter(|a| a.enabled)
            .map(|a| (a.exe.clone(), a.path.clone(), a.display.clone(), a.icon.clone(), a.auth_kind.clone()))
            .collect()
    };

    let mut rt = state.rt.lock().unwrap();

    // Olvida PIDs congelados que ya no existen.
    let alive: HashSet<u32> = live.iter().map(|p| p.pid).collect();
    rt.frozen.retain(|pid, _| alive.contains(pid));

    for (exe, path, display, icon, auth_kind) in apps {
        let key = exe.to_lowercase();
        let pids: Vec<u32> = live.iter().filter(|p| p.is(&exe)).map(|p| p.pid).collect();
        let running = !pids.is_empty();
        let shown = pids.iter().any(|p| visible.contains(p));

        // ¿Sigue permitida?
        let keep = match rt.allowed.get_mut(&key) {
            None => None,
            Some(Allow::Until(when)) => Some(Instant::now() < *when),
            Some(Allow::WhileShown { seen, hidden_since }) => Some(if !running {
                false
            } else if shown {
                *seen = true;
                *hidden_since = None;
                true
            } else if *seen {
                hidden_since.get_or_insert_with(Instant::now).elapsed() < HIDE_GRACE
            } else {
                true // aún no ha sacado ventana (o no tiene): sigue permitida
            }),
        };
        match keep {
            Some(true) => continue,
            Some(false) => {
                rt.allowed.remove(&key);
                // Caducó mientras está escondida: se bloquea cuando reaparezca.
                if running && !shown {
                    rt.dormant.insert(key.clone());
                }
            }
            None => {}
        }

        if !running {
            rt.dormant.remove(&key);
            continue;
        }
        if rt.dormant.contains(&key) {
            if !shown {
                continue; // sigue viva en la bandeja
            }
            rt.dormant.remove(&key);
        }

        // Bloqueada y en marcha: congela los PIDs nuevos.
        for pid in &pids {
            if !rt.frozen.contains_key(pid) {
                procctl::suspend_pid(*pid);
                rt.frozen.insert(*pid, key.clone());
            }
        }

        if rt.pending.is_some() {
            continue; // ya hay una petición abierta; esta espera congelada
        }
        rt.pending = Some(exe.clone());

        // Guarda la ruta real si aún no la teníamos.
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
        state.prompt_fails.lock().unwrap().remove(&key);

        let prompt = Prompt { exe: exe.clone(), display, icon, auth_kind };
        if let Some(win) = app.get_webview_window("unlock") {
            let _ = win.emit("lock:prompt", prompt.clone());
            let _ = win.show();
            let _ = win.set_focus();
            let _ = win.set_always_on_top(true);
        }
        let _ = app.emit("lock:prompt", prompt);
    }

    // La app de la petición abierta desapareció (la cerraron por otra vía).
    if let Some(p) = rt.pending.clone() {
        if !live.iter().any(|l| l.is(&p)) {
            rt.pending = None;
            if let Some(win) = app.get_webview_window("unlock") {
                let _ = win.hide();
            }
        }
    }
}
