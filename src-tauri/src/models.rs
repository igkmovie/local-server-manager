use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub cwd: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub health_url: Option<String>,
    #[serde(default)]
    pub open_url: Option<String>,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default)]
    pub start_timeout_sec: Option<u64>,
    #[serde(default)]
    pub stop_timeout_sec: Option<u64>,
    #[serde(default = "default_true")]
    pub graceful_stop: bool,
    #[serde(default)]
    pub shell: bool,
    #[serde(default)]
    pub log_file: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerGroupItem {
    pub server_id: String,
    pub order: i32,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerGroupConfig {
    pub id: String,
    pub name: String,
    pub servers: Vec<ServerGroupItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    #[serde(default)]
    pub servers: Vec<ServerConfig>,
    #[serde(default)]
    pub groups: Vec<ServerGroupConfig>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Unhealthy,
    Stopping,
    Error,
    External,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerView {
    pub id: String,
    pub name: String,
    pub status: ServerStatus,
    pub port: Option<u16>,
    pub health_url: Option<String>,
    pub open_url: Option<String>,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub pid: Option<u32>,
    pub last_started_at: Option<String>,
    pub last_stopped_at: Option<String>,
    pub last_exit_code: Option<i32>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupView {
    pub id: String,
    pub name: String,
    pub server_ids: Vec<String>,
}
