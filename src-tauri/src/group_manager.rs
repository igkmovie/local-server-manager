use crate::app_state::AppState;
use crate::models::{AppConfig, ServerStatus};
use crate::{compute_status, process_manager};

const DEPENDS_ON_WAIT_SEC: u64 = 60;

pub fn start_group(state: &AppState, cfg: &AppConfig, group_id: &str) -> Result<(), String> {
    let group = cfg
        .groups
        .iter()
        .find(|g| g.id == group_id)
        .ok_or_else(|| format!("Group not found: {}", group_id))?;

    let mut items = group.servers.clone();
    items.sort_by_key(|i| i.order);

    for item in items {
        for dep in &item.depends_on {
            wait_until_running(state, cfg, dep, DEPENDS_ON_WAIT_SEC).map_err(|e| {
                format!("Aborted at '{}': dependency '{}' not ready ({})", item.server_id, dep, e)
            })?;
        }

        let server = cfg
            .servers
            .iter()
            .find(|s| s.id == item.server_id)
            .ok_or_else(|| format!("Unknown server in group: {}", item.server_id))?;

        if process_manager::is_running(state, &server.id) {
            continue;
        }
        if let Some(p) = server.port {
            if crate::port_checker::is_port_in_use(p) {
                state.logs.append(
                    &server.id,
                    "system",
                    &format!(
                        "Skipped in group start: port {} already in use (external)",
                        p
                    ),
                );
                continue;
            }
        }

        process_manager::start(state, server)
            .map_err(|e| format!("Failed to start '{}': {}", server.id, e))?;
    }
    Ok(())
}

pub fn stop_group(state: &AppState, cfg: &AppConfig, group_id: &str) -> Vec<(String, String)> {
    let group = match cfg.groups.iter().find(|g| g.id == group_id) {
        Some(g) => g,
        None => return vec![(group_id.into(), "Group not found".into())],
    };

    let mut items = group.servers.clone();
    items.sort_by_key(|i| i.order);
    items.reverse();

    let mut errors = Vec::new();
    for item in items {
        if !process_manager::is_running(state, &item.server_id) {
            continue;
        }
        let server = match cfg.servers.iter().find(|s| s.id == item.server_id) {
            Some(s) => s,
            None => continue,
        };
        let timeout = server.stop_timeout_sec.unwrap_or(10);
        if let Err(e) = process_manager::stop(state, &server.id, timeout, server.graceful_stop) {
            errors.push((server.id.clone(), e));
        }
    }
    errors
}

fn wait_until_running(
    state: &AppState,
    cfg: &AppConfig,
    server_id: &str,
    timeout_sec: u64,
) -> Result<(), String> {
    let server = cfg
        .servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Unknown server: {}", server_id))?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_sec);
    loop {
        let status = compute_status(state, server);
        if status == ServerStatus::Running {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err(format!("timed out waiting for {}", server_id));
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
