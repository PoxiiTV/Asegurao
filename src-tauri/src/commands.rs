// Comandos que la interfaz (Svelte) invoca. Regla de fondo: reforzar la
// protección es fácil (solo sesión); debilitarla pide la contraseña maestra
// y además la confirmación de Windows (UAC).

use crate::state::{Allow, AppState};
use crate::store::{self, hash_secret, verify_secret, Auth, LockedApp, Settings};
use crate::{apps, history, procctl, uac};
use rand::Rng;
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::State;

#[derive(Serialize)]
pub struct AppView {
    pub display: String,
    pub exe: String,
    pub path: String,
    pub icon: String,
    pub auth_kind: String,
    pub trust_minutes: u32,
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct Snapshot {
    pub configured: bool,
    pub auth_kind: String,
    pub settings: Settings,
    pub apps: Vec<AppView>,
}

fn snapshot(state: &AppState) -> Snapshot {
    let data = state.data.lock().unwrap();
    Snapshot {
        configured: data.auth.is_some(),
        auth_kind: data.auth.as_ref().map(|a| a.kind.clone()).unwrap_or_default(),
        settings: data.settings.clone(),
        apps: data
            .apps
            .iter()
            .map(|a| AppView {
                display: a.display.clone(),
                exe: a.exe.clone(),
                path: a.path.clone(),
                icon: a.icon.clone(),
                auth_kind: a.auth_kind.clone(),
                trust_minutes: a.trust_minutes,
                enabled: a.enabled,
            })
            .collect(),
    }
}

fn gen_recovery_code() -> String {
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    let mut groups = Vec::new();
    for _ in 0..4 {
        let g: String = (0..4).map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char).collect();
        groups.push(g);
    }
    groups.join("-")
}

// ---------- Estado general ----------

#[tauri::command]
pub fn get_snapshot(state: State<Arc<AppState>>) -> Snapshot {
    snapshot(&state)
}

#[tauri::command]
pub fn session_status(state: State<Arc<AppState>>) -> bool {
    state.rt.lock().unwrap().session_unlocked
}

#[tauri::command]
pub fn touch(state: State<Arc<AppState>>) {
    state.rt.lock().unwrap().last_activity = Instant::now();
}

#[tauri::command]
pub fn lock_session(state: State<Arc<AppState>>) {
    state.rt.lock().unwrap().session_unlocked = false;
}

// ---------- Configuración inicial y sesión ----------

#[tauri::command]
pub fn setup(state: State<Arc<AppState>>, kind: String, secret: String) -> Result<String, String> {
    if secret.trim().is_empty() {
        return Err("La contraseña no puede estar vacía".into());
    }
    let code = gen_recovery_code();
    {
        let mut data = state.data.lock().unwrap();
        data.auth = Some(Auth {
            kind,
            hash: hash_secret(&secret),
            recovery_hash: hash_secret(&code),
        });
        data.settings = Settings::default();
        store::save(&data)?;
    }
    state.rt.lock().unwrap().session_unlocked = true;
    Ok(code)
}

#[tauri::command]
pub fn login(state: State<Arc<AppState>>, secret: String) -> Result<bool, String> {
    let mut rt = state.rt.lock().unwrap();
    if let Some(when) = rt.blocked_until {
        if Instant::now() < when {
            let secs = (when - Instant::now()).as_secs() + 1;
            return Err(format!("Demasiados intentos. Espera {secs} s"));
        }
    }
    let ok = {
        let data = state.data.lock().unwrap();
        data.auth.as_ref().map(|a| verify_secret(&secret, &a.hash)).unwrap_or(false)
    };
    if ok {
        rt.fails = 0;
        rt.blocked_until = None;
        rt.session_unlocked = true;
        rt.last_activity = Instant::now();
        Ok(true)
    } else {
        rt.fails += 1;
        if rt.fails >= 5 {
            let wait = (rt.fails - 4) as u64 * 30;
            rt.blocked_until = Some(Instant::now() + Duration::from_secs(wait));
        }
        Ok(false)
    }
}

#[tauri::command]
pub fn recover(state: State<Arc<AppState>>, code: String, kind: String, new_secret: String) -> Result<String, String> {
    if new_secret.trim().is_empty() {
        return Err("La nueva contraseña no puede estar vacía".into());
    }
    let mut data = state.data.lock().unwrap();
    let auth = data.auth.as_ref().ok_or("No configurado")?;
    if !verify_secret(&code.trim().to_uppercase(), &auth.recovery_hash) {
        return Err("Código de recuperación incorrecto".into());
    }
    let new_code = gen_recovery_code();
    data.auth = Some(Auth { kind, hash: hash_secret(&new_secret), recovery_hash: hash_secret(&new_code) });
    store::save(&data)?;
    Ok(new_code)
}

#[tauri::command]
pub fn change_master(state: State<Arc<AppState>>, current: String, kind: String, new_secret: String) -> Result<(), String> {
    {
        let data = state.data.lock().unwrap();
        let auth = data.auth.as_ref().ok_or("No configurado")?;
        if !verify_secret(&current, &auth.hash) {
            return Err("Contraseña actual incorrecta".into());
        }
    }
    if !uac::confirm() {
        return Err("Se necesita la confirmación de Windows".into());
    }
    let mut data = state.data.lock().unwrap();
    let recovery = data.auth.as_ref().unwrap().recovery_hash.clone();
    data.auth = Some(Auth { kind, hash: hash_secret(&new_secret), recovery_hash: recovery });
    store::save(&data)?;
    Ok(())
}

// ---------- Ajustes ----------

#[tauri::command]
pub fn set_theme(state: State<Arc<AppState>>, theme: String) -> Result<(), String> {
    let mut data = state.data.lock().unwrap();
    data.settings.theme = theme;
    store::save(&data)
}

#[tauri::command]
pub fn set_autostart(app: tauri::AppHandle, on: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let mgr = app.autolaunch();
    if on {
        mgr.enable().map_err(|e| e.to_string())
    } else {
        mgr.disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn save_settings(state: State<Arc<AppState>>, settings: Settings) -> Result<(), String> {
    let weakens = {
        let data = state.data.lock().unwrap();
        let cur = &data.settings;
        (cur.level == "fortaleza" && settings.level == "base")
            || (cur.autolock_minutes > 0 && settings.autolock_minutes == 0)
    };
    if weakens && !uac::confirm() {
        return Err("Se necesita la confirmación de Windows para reducir la protección".into());
    }
    let mut data = state.data.lock().unwrap();
    data.settings = settings;
    store::save(&data)
}

// ---------- Descubrimiento de apps ----------

#[tauri::command]
pub fn list_installed() -> Vec<apps::AppInfo> {
    apps::installed_apps()
}

#[tauri::command]
pub fn list_running() -> Vec<apps::AppInfo> {
    apps::running_apps()
}

#[tauri::command]
pub fn app_from_path(path: String) -> apps::AppInfo {
    let exe = path.rsplit(['\\', '/']).next().unwrap_or("").to_string();
    let display = exe.trim_end_matches(".exe").trim_end_matches(".EXE").to_string();
    let icon = procctl::icon_png_base64(&path).unwrap_or_default();
    apps::AppInfo { display, exe, path, icon }
}

// ---------- Gestión de apps bloqueadas (reforzar = solo sesión) ----------

#[tauri::command]
pub fn add_app(
    state: State<Arc<AppState>>,
    display: String,
    exe: String,
    path: String,
    icon: String,
    auth_kind: String,
    app_secret: String,
    trust_minutes: u32,
) -> Result<(), String> {
    let self_exe = std::env::current_exe().ok().and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string())).unwrap_or_default();
    if exe.trim().is_empty() {
        return Err("Falta el ejecutable".into());
    }
    if exe.eq_ignore_ascii_case(&self_exe) || exe.eq_ignore_ascii_case("asegurao.exe") {
        return Err("No puedes bloquear Asegurao".into());
    }
    let hash = if auth_kind == "own" {
        if app_secret.trim().len() < 4 {
            return Err("La contraseña de la app es demasiado corta".into());
        }
        hash_secret(&app_secret)
    } else {
        String::new()
    };
    let mut data = state.data.lock().unwrap();
    if data.apps.iter().any(|a| a.exe.eq_ignore_ascii_case(&exe)) {
        return Err("Esa app ya está protegida".into());
    }
    data.apps.push(LockedApp {
        exe,
        path,
        display,
        icon,
        auth_kind,
        hash,
        trust_minutes,
        enabled: true,
    });
    store::save(&data)
}

#[tauri::command]
pub fn set_app_trust(state: State<Arc<AppState>>, exe: String, minutes: u32) -> Result<(), String> {
    let mut data = state.data.lock().unwrap();
    if let Some(a) = data.apps.iter_mut().find(|a| a.exe.eq_ignore_ascii_case(&exe)) {
        a.trust_minutes = minutes;
    }
    store::save(&data)
}

#[tauri::command]
pub fn change_app_password(state: State<Arc<AppState>>, exe: String, auth_kind: String, app_secret: String) -> Result<(), String> {
    let mut data = state.data.lock().unwrap();
    if let Some(a) = data.apps.iter_mut().find(|a| a.exe.eq_ignore_ascii_case(&exe)) {
        a.auth_kind = auth_kind.clone();
        a.hash = if auth_kind == "own" {
            if app_secret.trim().len() < 4 {
                return Err("La contraseña de la app es demasiado corta".into());
            }
            hash_secret(&app_secret)
        } else {
            String::new()
        };
    }
    store::save(&data)
}

// ---------- Debilitar = maestra + UAC ----------

#[tauri::command]
pub fn set_app_enabled(state: State<Arc<AppState>>, exe: String, enabled: bool, master: String) -> Result<(), String> {
    if !enabled {
        {
            let data = state.data.lock().unwrap();
            let auth = data.auth.as_ref().ok_or("No configurado")?;
            if !verify_secret(&master, &auth.hash) {
                return Err("Contraseña maestra incorrecta".into());
            }
        }
        if !uac::confirm() {
            return Err("Se necesita la confirmación de Windows".into());
        }
    }
    let mut data = state.data.lock().unwrap();
    if let Some(a) = data.apps.iter_mut().find(|a| a.exe.eq_ignore_ascii_case(&exe)) {
        a.enabled = enabled;
    }
    store::save(&data)
}

#[tauri::command]
pub fn remove_app(state: State<Arc<AppState>>, exe: String, master: String) -> Result<(), String> {
    {
        let data = state.data.lock().unwrap();
        let auth = data.auth.as_ref().ok_or("No configurado")?;
        if !verify_secret(&master, &auth.hash) {
            return Err("Contraseña maestra incorrecta".into());
        }
    }
    if !uac::confirm() {
        return Err("Se necesita la confirmación de Windows".into());
    }
    {
        let mut data = state.data.lock().unwrap();
        data.apps.retain(|a| !a.exe.eq_ignore_ascii_case(&exe));
        store::save(&data)?;
    }
    state.rt.lock().unwrap().allowed.remove(&exe.to_lowercase());
    Ok(())
}

// ---------- Historial ----------

#[tauri::command]
pub fn get_history() -> Vec<history::Entry> {
    let mut list = history::load();
    list.reverse(); // más recientes primero
    list
}

#[tauri::command]
pub fn clear_history(state: State<Arc<AppState>>, master: String) -> Result<(), String> {
    {
        let data = state.data.lock().unwrap();
        let auth = data.auth.as_ref().ok_or("No configurado")?;
        if !verify_secret(&master, &auth.hash) {
            return Err("Contraseña maestra incorrecta".into());
        }
    }
    if !uac::confirm() {
        return Err("Se necesita la confirmación de Windows".into());
    }
    history::clear();
    Ok(())
}

// ---------- Puerta de desinstalación ----------

#[tauri::command]
pub fn gate_check(secret: String) -> bool {
    let data = store::load();
    let ok = data
        .auth
        .map(|a| verify_secret(&secret, &a.hash))
        .unwrap_or(false);
    if ok {
        std::process::exit(0); // el desinstalador continúa
    }
    false
}

#[tauri::command]
pub fn gate_cancel() {
    std::process::exit(1); // el desinstalador se cancela
}

// ---------- Flujo de desbloqueo (desde la ventana flotante) ----------

// El vigilante puede emitir "lock:prompt" antes de que la ventana cargue y el
// evento se pierde; la ventana pregunta al montarse si ya hay uno en curso.
#[tauri::command]
pub fn unlock_pending(state: State<Arc<AppState>>) -> Option<crate::watcher::Prompt> {
    let exe = state.rt.lock().unwrap().pending.clone()?;
    let data = state.data.lock().unwrap();
    let a = data.apps.iter().find(|a| a.exe.eq_ignore_ascii_case(&exe))?;
    Some(crate::watcher::Prompt {
        exe: a.exe.clone(),
        display: a.display.clone(),
        icon: a.icon.clone(),
        auth_kind: a.auth_kind.clone(),
    })
}

#[tauri::command]
pub fn unlock_attempt(state: State<Arc<AppState>>, exe: String, secret: String) -> Result<String, String> {
    let key = exe.to_lowercase();
    let (ok, display, trust) = {
        let data = state.data.lock().unwrap();
        let app = data.apps.iter().find(|a| a.exe.eq_ignore_ascii_case(&exe));
        match app {
            Some(a) => {
                let ok = if a.auth_kind == "master" {
                    data.auth.as_ref().map(|m| verify_secret(&secret, &m.hash)).unwrap_or(false)
                } else {
                    verify_secret(&secret, &a.hash)
                };
                (ok, a.display.clone(), a.trust_minutes)
            }
            None => (false, exe.clone(), 0),
        }
    };

    if ok {
        procctl::resume(&exe);
        let allow = if trust == 0 {
            Allow::UntilExit
        } else {
            Allow::Until(Instant::now() + Duration::from_secs(trust as u64 * 60))
        };
        let mut rt = state.rt.lock().unwrap();
        rt.allowed.insert(key.clone(), allow);
        rt.pending = None;
        drop(rt);
        state.prompt_fails.lock().unwrap().remove(&key);
        history::add(history::Entry { ts: 0, exe, display, result: "allowed".into(), photo: String::new() });
        Ok("ok".into())
    } else {
        let n = {
            let mut fails = state.prompt_fails.lock().unwrap();
            let n = fails.entry(key.clone()).or_insert(0);
            *n += 1;
            *n
        };
        // Foto del intruso al alcanzar el umbral (una sola vez).
        let (enabled, after) = {
            let data = state.data.lock().unwrap();
            (data.settings.intruder.enabled, data.settings.intruder.after_fails)
        };
        if enabled && n == after {
            let ex = exe.clone();
            std::thread::spawn(move || {
                let photo = crate::intruder::capture().unwrap_or_default();
                history::add(history::Entry {
                    ts: 0,
                    exe: ex.clone(),
                    display,
                    result: "tamper".into(),
                    photo,
                });
            });
        }
        Ok("wrong".into())
    }
}

#[tauri::command]
pub fn unlock_cancel(state: State<Arc<AppState>>, exe: String) {
    let key = exe.to_lowercase();
    let display = {
        let data = state.data.lock().unwrap();
        data.apps.iter().find(|a| a.exe.eq_ignore_ascii_case(&exe)).map(|a| a.display.clone()).unwrap_or_else(|| exe.clone())
    };
    // ponytail: foto del intruso se captura aquí cuando se supera el umbral (2ª fase).
    procctl::terminate(&exe);
    let mut rt = state.rt.lock().unwrap();
    rt.pending = None;
    rt.allowed.remove(&key);
    drop(rt);
    state.prompt_fails.lock().unwrap().remove(&key);
    history::add(history::Entry { ts: 0, exe, display, result: "denied".into(), photo: String::new() });
}
