// Almacén persistente: config.json en %APPDATA%\Asegurao, hashes con Argon2id,
// escritura atómica (tmp + rename) para no corromper si se corta la luz.

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    let dir = PathBuf::from(base).join("Asegurao");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn config_path() -> PathBuf {
    data_dir().join("config.json")
}

pub fn hash_secret(secret: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map(|h| h.to_string())
        .unwrap_or_default()
}

pub fn verify_secret(secret: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(secret.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Auth {
    pub kind: String,          // "pin" | "password"
    pub hash: String,          // argon2 de la contraseña maestra
    pub recovery_hash: String, // argon2 del código de recuperación
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Intruder {
    pub enabled: bool,
    pub after_fails: u32,
}

impl Default for Intruder {
    fn default() -> Self {
        Intruder { enabled: false, after_fails: 3 }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub theme: String,           // "grafito" | "boveda" | "aurora"
    pub autolock_minutes: u32,   // 0 = nunca
    pub level: String,           // "base" | "fortaleza"
    pub intruder: Intruder,
    pub start_with_windows: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "grafito".into(),
            autolock_minutes: 5,
            level: "fortaleza".into(),
            intruder: Intruder::default(),
            start_with_windows: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LockedApp {
    pub exe: String,           // nombre del ejecutable, ej "Discord.exe"
    pub path: String,          // ruta completa conocida (puede estar vacía)
    pub display: String,       // nombre visible
    pub icon: String,          // PNG en base64 (sin prefijo data:)
    pub auth_kind: String,     // "own" (contraseña propia) | "master" (usa la maestra)
    pub hash: String,          // argon2 si auth_kind == "own"
    pub trust_minutes: u32,    // "tiempo de confianza": 0/5/15/60
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppData {
    #[serde(default)]
    pub auth: Option<Auth>,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub apps: Vec<LockedApp>,
}

pub fn load() -> AppData {
    match fs::read_to_string(config_path()) {
        Ok(txt) => serde_json::from_str(&txt).unwrap_or_default(),
        Err(_) => AppData::default(),
    }
}

pub fn save(data: &AppData) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    let tmp = config_path().with_extension("json.tmp");
    fs::write(&tmp, json.as_bytes()).map_err(|e| e.to_string())?;
    fs::rename(&tmp, config_path()).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_roundtrip() {
        let h = hash_secret("123456");
        assert!(verify_secret("123456", &h));
        assert!(!verify_secret("000000", &h));
    }
}
