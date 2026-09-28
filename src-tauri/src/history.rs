// Historial de intentos (máx. 1000, rota solo) y carpeta de fotos del intruso.

use crate::store::data_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_ENTRIES: usize = 1000;

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    pub ts: u64,          // epoch segundos
    pub exe: String,
    pub display: String,
    pub result: String,   // "allowed" | "denied" | "tamper"
    pub photo: String,    // nombre de archivo en fotos/ o vacío
}

fn history_path() -> PathBuf {
    data_dir().join("history.json")
}

pub fn photos_dir() -> PathBuf {
    let dir = data_dir().join("fotos");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

pub fn load() -> Vec<Entry> {
    match fs::read_to_string(history_path()) {
        Ok(txt) => serde_json::from_str(&txt).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn add(mut entry: Entry) {
    let mut list = load();
    if entry.ts == 0 {
        entry.ts = now();
    }
    list.push(entry);
    let len = list.len();
    if len > MAX_ENTRIES {
        list.drain(0..len - MAX_ENTRIES);
    }
    if let Ok(json) = serde_json::to_string(&list) {
        let _ = fs::write(history_path(), json);
    }
}

pub fn clear() {
    let _ = fs::remove_file(history_path());
    if let Ok(entries) = fs::read_dir(photos_dir()) {
        for e in entries.flatten() {
            let _ = fs::remove_file(e.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_caps_length() {
        let mut list: Vec<Entry> = (0..MAX_ENTRIES + 50)
            .map(|i| Entry { ts: i as u64, exe: "x".into(), display: "x".into(), result: "denied".into(), photo: String::new() })
            .collect();
        let len = list.len();
        if len > MAX_ENTRIES {
            list.drain(0..len - MAX_ENTRIES);
        }
        assert_eq!(list.len(), MAX_ENTRIES);
        assert_eq!(list[0].ts, 50);
    }
}
