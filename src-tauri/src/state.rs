// Estado en memoria compartido entre la interfaz, los comandos y el vigilante.
// Los tiempos usan Instant (reloj monotónico): cambiar la hora de Windows no ayuda.

use crate::store::AppData;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::Instant;

pub enum Allow {
    // "Sin confianza": permitida mientras se vea. Si se esconde en la bandeja
    // (tras haber tenido ventana), se vuelve a bloquear al reaparecer.
    WhileShown { seen: bool, hidden_since: Option<Instant> },
    Until(Instant), // "tiempo de confianza": permitida hasta este momento
}

pub struct Runtime {
    pub session_unlocked: bool,
    pub last_activity: Instant,
    pub allowed: HashMap<String, Allow>, // clave: exe en minúsculas
    pub pending: Option<String>,         // exe con diálogo de desbloqueo abierto
    pub frozen: HashMap<u32, String>,    // PID congelado -> exe en minúsculas
    pub dormant: HashSet<String>,        // escondidas en la bandeja: se bloquean al reaparecer
    pub fails: u32,                      // fallos consecutivos en el login
    pub blocked_until: Option<Instant>,  // espera tras varios fallos
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            session_unlocked: false,
            last_activity: Instant::now(),
            allowed: HashMap::new(),
            pending: None,
            frozen: HashMap::new(),
            dormant: HashSet::new(),
            fails: 0,
            blocked_until: None,
        }
    }
}

impl Runtime {
    /// Suelta los PIDs congelados de una app: los reanuda o, si `kill`, los cierra.
    pub fn release(&mut self, key: &str, kill: bool) {
        self.frozen.retain(|pid, k| {
            if k != key {
                return true;
            }
            if kill {
                crate::procctl::terminate_pid(*pid);
            } else {
                crate::procctl::resume_pid(*pid);
            }
            false
        });
        if self.pending.as_deref().is_some_and(|p| p.eq_ignore_ascii_case(key)) {
            self.pending = None;
        }
    }

    /// Reanuda todo lo congelado (antes de cerrar o actualizar Asegurao).
    pub fn release_all(&mut self) {
        for pid in self.frozen.keys() {
            crate::procctl::resume_pid(*pid);
        }
        self.frozen.clear();
        self.pending = None;
    }
}

pub struct AppState {
    pub data: Mutex<AppData>,
    pub rt: Mutex<Runtime>,
    pub prompt_fails: Mutex<HashMap<String, u32>>, // fallos por app en el diálogo de desbloqueo
}

impl AppState {
    pub fn new(data: AppData) -> Self {
        AppState {
            data: Mutex::new(data),
            rt: Mutex::new(Runtime::default()),
            prompt_fails: Mutex::new(HashMap::new()),
        }
    }
}
