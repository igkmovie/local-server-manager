mod app_state;
mod config;
mod group_manager;
mod health_checker;
mod log_manager;
mod models;
mod port_checker;
mod process_manager;

use app_state::AppState;
use models::{GroupView, ServerConfig, ServerStatus, ServerView};
use tauri::State;

#[tauri::command]
fn ping() -> String {
    "pong from rust".to_string()
}

fn load_config_into(state: &AppState) -> Result<models::AppConfig, String> {
    let cfg = config::load_or_init()?;
    *state.config.lock().unwrap() = Some(cfg.clone());
    Ok(cfg)
}

pub fn compute_status(state: &AppState, s: &ServerConfig) -> ServerStatus {
    let managed = process_manager::is_running(state, &s.id);
    if managed {
        let port_ok = match s.port {
            Some(p) => port_checker::is_port_in_use(p),
            None => true,
        };
        let health_ok = match &s.health_url {
            Some(u) => health_checker::check(u),
            None => true,
        };
        if port_ok && health_ok {
            ServerStatus::Running
        } else {
            ServerStatus::Unhealthy
        }
    } else if let Some(p) = s.port {
        if port_checker::is_port_in_use(p) {
            ServerStatus::External
        } else {
            ServerStatus::Stopped
        }
    } else {
        ServerStatus::Stopped
    }
}

#[tauri::command]
fn get_servers(state: State<'_, AppState>) -> Result<Vec<ServerView>, String> {
    let cfg = load_config_into(&state)?;
    Ok(cfg
        .servers
        .into_iter()
        .map(|s| {
            let status = compute_status(&state, &s);
            ServerView {
                status,
                pid: process_manager::get_pid(&state, &s.id),
                last_started_at: process_manager::started_at(&state, &s.id),
                id: s.id,
                name: s.name,
                port: s.port,
                health_url: s.health_url,
                open_url: s.open_url,
                command: s.command,
                args: s.args,
                cwd: s.cwd,
                last_stopped_at: None,
                last_exit_code: None,
                last_error: None,
            }
        })
        .collect())
}

#[tauri::command]
fn get_groups(state: State<'_, AppState>) -> Result<Vec<GroupView>, String> {
    let cfg = load_config_into(&state)?;
    Ok(cfg
        .groups
        .into_iter()
        .map(|g| {
            let mut items = g.servers;
            items.sort_by_key(|i| i.order);
            GroupView {
                id: g.id,
                name: g.name,
                server_ids: items.into_iter().map(|i| i.server_id).collect(),
            }
        })
        .collect())
}

#[tauri::command]
fn start_server(state: State<'_, AppState>, server_id: String) -> Result<u32, String> {
    let cfg = load_config_into(&state)?;
    let server = cfg
        .servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server not found: {}", server_id))?;
    if let Some(p) = server.port {
        if !process_manager::is_running(&state, &server.id) && port_checker::is_port_in_use(p) {
            return Err(format!(
                "Port {} is already in use by another process. This server may already be running outside Local Server Manager.",
                p
            ));
        }
    }
    process_manager::start(&state, server)
}

fn find_server(cfg: &models::AppConfig, id: &str) -> Result<ServerConfig, String> {
    cfg.servers
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| format!("Server not found: {}", id))
}

#[tauri::command]
fn stop_server(state: State<'_, AppState>, server_id: String) -> Result<(), String> {
    let cfg = load_config_into(&state)?;
    let server = find_server(&cfg, &server_id)?;
    let timeout = server.stop_timeout_sec.unwrap_or(10);
    process_manager::stop(&state, &server_id, timeout, server.graceful_stop)
}

#[tauri::command]
fn restart_server(state: State<'_, AppState>, server_id: String) -> Result<u32, String> {
    let cfg = load_config_into(&state)?;
    let server = find_server(&cfg, &server_id)?;
    if process_manager::is_running(&state, &server_id) {
        let timeout = server.stop_timeout_sec.unwrap_or(10);
        process_manager::stop(&state, &server_id, timeout, server.graceful_stop)?;
    }
    process_manager::start(&state, &server)
}

#[tauri::command]
fn start_group(state: State<'_, AppState>, group_id: String) -> Result<(), String> {
    let cfg = load_config_into(&state)?;
    group_manager::start_group(&state, &cfg, &group_id)
}

#[tauri::command]
fn stop_group(state: State<'_, AppState>, group_id: String) -> Result<Vec<(String, String)>, String> {
    let cfg = load_config_into(&state)?;
    Ok(group_manager::stop_group(&state, &cfg, &group_id))
}

#[tauri::command]
fn get_server_log(server_id: String, lines: Option<usize>) -> Result<String, String> {
    log_manager::read_tail(&server_id, lines.unwrap_or(500))
}

#[tauri::command]
fn open_server_url(state: State<'_, AppState>, app: tauri::AppHandle, server_id: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let cfg = load_config_into(&state)?;
    let server = find_server(&cfg, &server_id)?;
    let url = server
        .open_url
        .or_else(|| server.health_url.clone())
        .ok_or_else(|| format!("'{}' has no openUrl configured", server_id))?;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn reveal_log_file(app: tauri::AppHandle, server_id: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    config::ensure_dirs()?;
    let log_path = config::logs_dir()?.join(format!("{}.log", server_id));
    let target = if log_path.exists() {
        log_path
    } else {
        config::logs_dir()?
    };
    app.opener()
        .reveal_item_in_dir(&target)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config_path() -> Result<String, String> {
    Ok(config::config_path()?.to_string_lossy().to_string())
}

#[tauri::command]
fn reveal_config_file(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    config::ensure_dirs()?;
    let parent = config::app_data_dir()?.to_string_lossy().to_string();
    app.opener()
        .open_path(parent, None::<&str>)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            ping,
            get_servers,
            get_groups,
            start_server,
            stop_server,
            restart_server,
            start_group,
            stop_group,
            get_server_log,
            open_server_url,
            reveal_log_file,
            get_config_path,
            reveal_config_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
