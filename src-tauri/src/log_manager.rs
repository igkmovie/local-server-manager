use crate::config;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BYTES: u64 = 10 * 1024 * 1024;

pub struct LogWriter {
    file: File,
    path: PathBuf,
    bytes: u64,
}

impl LogWriter {
    fn open(server_id: &str) -> Result<Self, String> {
        let dir = config::logs_dir()?;
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("{}.log", server_id));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        let bytes = file.metadata().map(|m| m.len()).unwrap_or(0);
        Ok(Self { file, path, bytes })
    }

    fn write_line(&mut self, kind: &str, line: &str) {
        let ts = format_ts();
        let entry = format!("[{}] [{}] {}\n", ts, kind, line);
        let bytes = entry.as_bytes();
        if let Err(e) = self.file.write_all(bytes) {
            eprintln!("log write failed: {}", e);
            return;
        }
        self.bytes += bytes.len() as u64;
        if self.bytes > MAX_BYTES {
            self.rotate();
        }
    }

    fn rotate(&mut self) {
        let rotated = self.path.with_extension("log.1");
        let _ = fs::remove_file(&rotated);
        if fs::rename(&self.path, &rotated).is_ok() {
            if let Ok(f) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)
            {
                self.file = f;
                self.bytes = 0;
            }
        }
    }
}

#[derive(Default)]
pub struct LogRegistry {
    writers: Mutex<HashMap<String, Mutex<LogWriter>>>,
}

impl LogRegistry {
    pub fn append(&self, server_id: &str, kind: &str, line: &str) {
        let mut map = self.writers.lock().unwrap();
        let writer = match map.get(server_id) {
            Some(w) => w,
            None => {
                let w = match LogWriter::open(server_id) {
                    Ok(w) => w,
                    Err(e) => {
                        eprintln!("log open failed for {}: {}", server_id, e);
                        return;
                    }
                };
                map.insert(server_id.to_string(), Mutex::new(w));
                map.get(server_id).unwrap()
            }
        };
        writer.lock().unwrap().write_line(kind, line);
    }
}

pub fn read_tail(server_id: &str, lines: usize) -> Result<String, String> {
    let path = config::logs_dir()?.join(format!("{}.log", server_id));
    if !path.exists() {
        return Ok(String::new());
    }
    let file = File::open(&path).map_err(|e| e.to_string())?;
    let size = file.metadata().map_err(|e| e.to_string())?.len();
    let approx = (lines as u64) * 256;
    let start = size.saturating_sub(approx);
    let mut reader = BufReader::new(file);
    if start > 0 {
        reader
            .seek(SeekFrom::Start(start))
            .map_err(|e| e.to_string())?;
        let mut discard = String::new();
        let _ = reader.read_line(&mut discard);
    }
    let mut all: Vec<String> = Vec::new();
    for line in reader.lines().map_while(Result::ok) {
        all.push(line);
    }
    let take = all.len().min(lines);
    let tail = &all[all.len() - take..];
    Ok(tail.join("\n"))
}

fn format_ts() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, h, mi, s) = unix_to_ymdhms(secs);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, h, mi, s)
}

fn unix_to_ymdhms(secs: u64) -> (i32, u32, u32, u32, u32, u32) {
    let s = (secs % 60) as u32;
    let m = ((secs / 60) % 60) as u32;
    let h = ((secs / 3600) % 24) as u32;
    let days = (secs / 86_400) as i64;
    let (y, mo, d) = civil_from_days(days);
    (y, mo, d, h, m, s)
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = (y + if m <= 2 { 1 } else { 0 }) as i32;
    (year, m, d)
}
