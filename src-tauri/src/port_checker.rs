use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub fn is_port_in_use(port: u16) -> bool {
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok()
}

#[cfg(windows)]
pub fn find_pid_on_port(port: u16) -> Option<u32> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let out = Command::new("netstat")
        .arg("-ano")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let needle = format!(":{}", port);
    for line in stdout.lines() {
        let line = line.trim();
        if !line.starts_with("TCP") {
            continue;
        }
        if !line.contains("LISTENING") {
            continue;
        }
        let mut parts = line.split_whitespace();
        let _proto = parts.next();
        let local = match parts.next() {
            Some(s) => s,
            None => continue,
        };
        if !local.ends_with(&needle) {
            continue;
        }
        if let Some(pid_str) = line.split_whitespace().last() {
            if let Ok(pid) = pid_str.parse::<u32>() {
                return Some(pid);
            }
        }
    }
    None
}

#[cfg(not(windows))]
pub fn find_pid_on_port(_port: u16) -> Option<u32> {
    None
}
