use crate::app_state::{AppState, ManagedProcess};
use crate::log_manager::LogRegistry;
use crate::models::ServerConfig;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn now_iso() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", now)
}

pub fn is_running(state: &AppState, server_id: &str) -> bool {
    let mut procs = state.processes.lock().unwrap();
    if let Some(mp) = procs.get_mut(server_id) {
        match mp.child.try_wait() {
            Ok(Some(status)) => {
                let code = status.code().unwrap_or(-1);
                state.logs.append(
                    server_id,
                    "system",
                    &format!("Process exited with code {}", code),
                );
                procs.remove(server_id);
                false
            }
            Ok(None) => true,
            Err(_) => true,
        }
    } else {
        false
    }
}

pub fn get_pid(state: &AppState, server_id: &str) -> Option<u32> {
    let procs = state.processes.lock().unwrap();
    procs.get(server_id).map(|mp| mp.pid)
}

pub fn started_at(state: &AppState, server_id: &str) -> Option<String> {
    let procs = state.processes.lock().unwrap();
    procs.get(server_id).map(|mp| mp.started_at.clone())
}

pub fn start(state: &AppState, cfg: &ServerConfig) -> Result<u32, String> {
    if is_running(state, &cfg.id) {
        return Err(format!("Server '{}' is already running", cfg.id));
    }

    let cwd = Path::new(&cfg.cwd);
    if !cwd.exists() {
        let msg = format!("Working directory does not exist: {}", cfg.cwd);
        state.logs.append(&cfg.id, "system", &msg);
        return Err(msg);
    }

    let mut command = if cfg.shell {
        #[cfg(windows)]
        {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(&cfg.command).args(&cfg.args);
            c
        }
        #[cfg(not(windows))]
        {
            let mut joined = cfg.command.clone();
            for a in &cfg.args {
                joined.push(' ');
                joined.push_str(a);
            }
            let mut c = Command::new("sh");
            c.arg("-c").arg(joined);
            c
        }
    } else {
        let mut c = Command::new(&cfg.command);
        c.args(&cfg.args);
        c
    };

    command
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for (k, v) in &cfg.env {
        command.env(k, v);
    }

    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }

    state
        .logs
        .append(&cfg.id, "system", &format!("Starting: {} {:?}", cfg.command, cfg.args));

    let mut child = command.spawn().map_err(|e| {
        let msg = if e.kind() == std::io::ErrorKind::NotFound {
            format!(
                "Command not found: {}\n\nPlease check your PATH or use an absolute path.",
                cfg.command
            )
        } else {
            format!("Failed to start '{}': {}", cfg.command, e)
        };
        state.logs.append(&cfg.id, "system", &msg);
        msg
    })?;

    let pid = child.id();
    state
        .logs
        .append(&cfg.id, "system", &format!("Started with PID {}", pid));

    let logs = state.logs.clone();
    if let Some(out) = child.stdout.take() {
        spawn_reader(out, logs.clone(), cfg.id.clone(), "stdout");
    }
    if let Some(err) = child.stderr.take() {
        spawn_reader(err, logs.clone(), cfg.id.clone(), "stderr");
    }

    let mp = ManagedProcess {
        child,
        pid,
        started_at: now_iso(),
    };
    state.processes.lock().unwrap().insert(cfg.id.clone(), mp);
    Ok(pid)
}

pub fn stop(state: &AppState, server_id: &str, timeout_sec: u64, graceful: bool) -> Result<(), String> {
    let pid = {
        let procs = state.processes.lock().unwrap();
        match procs.get(server_id) {
            Some(mp) => mp.pid,
            None => return Err(format!("Server '{}' is not managed by this app", server_id)),
        }
    };

    state.logs.append(server_id, "system", "Stop requested");

    if graceful {
        let _ = run_taskkill(pid, false);
        if wait_for_exit(state, server_id, timeout_sec) {
            state.logs.append(server_id, "system", "Stopped gracefully");
            state.processes.lock().unwrap().remove(server_id);
            return Ok(());
        }
        state
            .logs
            .append(server_id, "system", "Graceful stop timed out, forcing kill");
    }

    match run_taskkill(pid, true) {
        Ok(_) => {}
        Err(e) => {
            state
                .logs
                .append(server_id, "system", &format!("taskkill failed: {}", e));
        }
    }

    if !wait_for_exit(state, server_id, 5) {
        state
            .logs
            .append(server_id, "system", "Process did not exit after force kill");
        return Err("Process did not exit after force kill".into());
    }

    state.logs.append(server_id, "system", "Stopped (forced)");
    state.processes.lock().unwrap().remove(server_id);
    Ok(())
}

fn wait_for_exit(state: &AppState, server_id: &str, timeout_sec: u64) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_sec);
    while std::time::Instant::now() < deadline {
        {
            let mut procs = state.processes.lock().unwrap();
            if let Some(mp) = procs.get_mut(server_id) {
                if let Ok(Some(_)) = mp.child.try_wait() {
                    return true;
                }
            } else {
                return true;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    false
}

#[cfg(windows)]
fn run_taskkill(pid: u32, force: bool) -> Result<(), String> {
    let mut cmd = Command::new("taskkill");
    cmd.arg("/PID").arg(pid.to_string()).arg("/T");
    if force {
        cmd.arg("/F");
    }
    cmd.creation_flags(CREATE_NO_WINDOW);
    let status = cmd.status().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("taskkill exit code: {:?}", status.code()))
    }
}

#[cfg(not(windows))]
fn run_taskkill(pid: u32, force: bool) -> Result<(), String> {
    let sig = if force { "-KILL" } else { "-TERM" };
    let status = Command::new("kill")
        .arg(sig)
        .arg(pid.to_string())
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("kill exit code: {:?}", status.code()))
    }
}

fn spawn_reader<R: std::io::Read + Send + 'static>(
    reader: R,
    logs: Arc<LogRegistry>,
    server_id: String,
    kind: &'static str,
) {
    thread::spawn(move || {
        let buf = BufReader::new(reader);
        for line in buf.lines().map_while(Result::ok) {
            logs.append(&server_id, kind, &line);
        }
    });
}
