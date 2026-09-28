// Estado en memoria compartido entre la interfaz, los comandos y el vigilante.
// Los tiempos usan Instant (reloj monotónico): cambiar la hora de Windows no ayuda.

use crate::store::AppData;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

pub enum Allow {
    UntilExit,       // "sin confianza": permitido solo mientras siga abierta
    Until(Instant),  // "tiempo de confianza": permitido hasta este momento
}

pub struct Runtime {
    pub session_unlocked: bool,
    pub last_activity: Instant,
    pub allowed: HashMap<String, Allow>, // clave: exe en minúsculas
    pub pending: Option<String>,         // exe con diálogo de desbloqueo abierto
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
            fails: 0,
            blocked_until: None,
        }
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
