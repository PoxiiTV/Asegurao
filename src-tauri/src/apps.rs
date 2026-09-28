// Descubrimiento de aplicaciones: instaladas (registro) y abiertas ahora mismo.

use crate::procctl;
use serde::Serialize;
use std::collections::BTreeMap;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Serialize, Clone)]
pub struct AppInfo {
    pub display: String,
    pub exe: String,
    pub path: String,
    pub icon: String, // PNG base64 (puede ir vacío)
}

fn clean_icon_path(raw: &str) -> String {
    raw.split(',').next().unwrap_or("").replace('"', "").trim().to_string()
}

fn exe_from_path(path: &str) -> String {
    path.rsplit(['\\', '/']).next().unwrap_or("").to_string()
}

fn scan_uninstall(root: RegKey, subpath: &str, out: &mut BTreeMap<String, AppInfo>) {
    let key = match root.open_subkey(subpath) {
        Ok(k) => k,
        Err(_) => return,
    };
    for name in key.enum_keys().flatten() {
        let sub = match key.open_subkey(&name) {
            Ok(k) => k,
            Err(_) => continue,
        };
        let display: String = sub.get_value("DisplayName").unwrap_or_default();
        let display_icon: String = sub.get_value("DisplayIcon").unwrap_or_default();
        if display.is_empty() || display_icon.is_empty() {
            continue;
        }
        let path = clean_icon_path(&display_icon);
        let exe = exe_from_path(&path);
        if !exe.to_lowercase().ends_with(".exe") {
            continue;
        }
        out.entry(display.clone()).or_insert(AppInfo {
            display,
            exe,
            path,
            icon: String::new(),
        });
    }
}

pub fn installed_apps() -> Vec<AppInfo> {
    let mut map = BTreeMap::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    scan_uninstall(hklm, r"Software\Microsoft\Windows\CurrentVersion\Uninstall", &mut map);
    // ponytail: icono por app extraído aquí; si con muchas apps se nota lento, hacerlo perezoso.
    scan_uninstall(
        RegKey::predef(HKEY_LOCAL_MACHINE),
        r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        &mut map,
    );
    scan_uninstall(hkcu, r"Software\Microsoft\Windows\CurrentVersion\Uninstall", &mut map);

    let mut list: Vec<AppInfo> = map.into_values().collect();
    for app in list.iter_mut() {
        if let Some(png) = procctl::icon_png_base64(&app.path) {
            app.icon = png;
        }
    }
    list
}

pub fn running_apps() -> Vec<AppInfo> {
    let mut map: BTreeMap<String, AppInfo> = BTreeMap::new();
    for proc in procctl::list_processes() {
        if !proc.exe.to_lowercase().ends_with(".exe") {
            continue;
        }
        if map.contains_key(&proc.exe.to_lowercase()) {
            continue;
        }
        let path = procctl::process_path(proc.pid).unwrap_or_default();
        let icon = if path.is_empty() {
            String::new()
        } else {
            procctl::icon_png_base64(&path).unwrap_or_default()
        };
        map.insert(
            proc.exe.to_lowercase(),
            AppInfo {
                display: proc.exe.trim_end_matches(".exe").trim_end_matches(".EXE").to_string(),
                exe: proc.exe.clone(),
                path,
                icon,
            },
        );
    }
    map.into_values().collect()
}
