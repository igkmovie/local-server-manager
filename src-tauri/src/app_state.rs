use crate::log_manager::LogRegistry;
use crate::models::AppConfig;
use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, Mutex};

pub struct ManagedProcess {
    pub child: Child,
    pub pid: u32,
    pub started_at: String,
}

#[derive(Default)]
pub struct AppState {
    pub config: Mutex<Option<AppConfig>>,
    pub processes: Mutex<HashMap<String, ManagedProcess>>,
    pub logs: Arc<LogRegistry>,
}
