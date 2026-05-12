use crate::models::{AppConfig, ServerConfig, ServerGroupConfig, ServerGroupItem};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

const APP_DIR_NAME: &str = "LocalServerManager";

pub fn app_data_dir() -> Result<PathBuf, String> {
    let base = dirs::config_dir().ok_or_else(|| "Could not resolve config dir".to_string())?;
    Ok(base.join(APP_DIR_NAME))
}

pub fn config_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("config.json"))
}

pub fn logs_dir() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("logs"))
}

pub fn ensure_dirs() -> Result<(), String> {
    fs::create_dir_all(app_data_dir()?).map_err(|e| e.to_string())?;
    fs::create_dir_all(logs_dir()?).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_or_init() -> Result<AppConfig, String> {
    ensure_dirs()?;
    let path = config_path()?;
    if !path.exists() {
        let sample = default_sample();
        let json = serde_json::to_string_pretty(&sample).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;
        return Ok(sample);
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;
    let cfg: AppConfig =
        serde_json::from_str(&text).map_err(|e| format!("Invalid config JSON: {}", e))?;
    validate(&cfg)?;
    Ok(cfg)
}

fn validate(cfg: &AppConfig) -> Result<(), String> {
    let mut ids: HashSet<&str> = HashSet::new();
    for s in &cfg.servers {
        if s.id.is_empty() {
            return Err("Server id is empty".into());
        }
        if !s
            .id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(format!("Server id '{}' contains invalid characters", s.id));
        }
        if !ids.insert(s.id.as_str()) {
            return Err(format!("Duplicate server id: {}", s.id));
        }
        if s.command.is_empty() {
            return Err(format!("Server '{}' has empty command", s.id));
        }
    }
    let server_ids: HashSet<&str> = cfg.servers.iter().map(|s| s.id.as_str()).collect();
    let mut group_ids: HashSet<&str> = HashSet::new();
    for g in &cfg.groups {
        if !group_ids.insert(g.id.as_str()) {
            return Err(format!("Duplicate group id: {}", g.id));
        }
        for item in &g.servers {
            if !server_ids.contains(item.server_id.as_str()) {
                return Err(format!(
                    "Group '{}' references unknown server '{}'",
                    g.id, item.server_id
                ));
            }
        }
    }
    Ok(())
}

fn srv(
    id: &str,
    name: &str,
    command: &str,
    args: &[&str],
    cwd: &str,
    port: Option<u16>,
    url: Option<&str>,
    shell: bool,
) -> ServerConfig {
    ServerConfig {
        id: id.to_string(),
        name: name.to_string(),
        command: command.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
        cwd: cwd.to_string(),
        env: HashMap::new(),
        port,
        health_url: url.map(String::from),
        open_url: url.map(String::from),
        auto_start: false,
        start_timeout_sec: Some(60),
        stop_timeout_sec: Some(10),
        graceful_stop: true,
        shell,
        log_file: Some(format!("{}.log", id)),
    }
}

fn group(id: &str, name: &str, server_ids: &[&str]) -> ServerGroupConfig {
    ServerGroupConfig {
        id: id.to_string(),
        name: name.to_string(),
        servers: server_ids
            .iter()
            .enumerate()
            .map(|(i, sid)| ServerGroupItem {
                server_id: (*sid).to_string(),
                order: (i + 1) as i32,
                depends_on: vec![],
            })
            .collect(),
    }
}

fn default_sample() -> AppConfig {
    AppConfig {
        version: 1,
        servers: vec![
            srv(
                "patreon-update",
                "Patreon Update Runner",
                "node",
                &["creaflow-patreon-runner.js"],
                "E:/patreon_update",
                None,
                None,
                false,
            ),
            srv(
                "serifugen",
                "SerifuGen",
                "npm",
                &["run", "dev"],
                "E:/SerifuGen",
                Some(7432),
                Some("http://127.0.0.1:7432"),
                true,
            ),
            srv(
                "creaflow",
                "CreaFlow",
                "npm",
                &["run", "dev"],
                "E:/CreaFlow/creaflow-app",
                Some(3333),
                Some("http://127.0.0.1:3333"),
                true,
            ),
            srv(
                "crearise",
                "CreaRise",
                "npm",
                &["run", "dev"],
                "E:/CreaRise",
                Some(4000),
                Some("http://127.0.0.1:4000"),
                true,
            ),
            srv(
                "silhouette-name-builder",
                "Silhouette Name Builder",
                "npm",
                &["run", "dev"],
                "E:/silhouette_name_builder",
                Some(18763),
                Some("http://127.0.0.1:18763"),
                true,
            ),
        ],
        groups: vec![group(
            "all-web",
            "All Web Apps",
            &["serifugen", "creaflow", "crearise", "silhouette-name-builder"],
        )],
    }
}
