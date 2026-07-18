use crate::config::types::VarValue;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use sysinfo::{CpuRefreshKind, System};

static SYS: OnceLock<Mutex<System>> = OnceLock::new();
static ACTIVESONG: Mutex<Option<String>> = Mutex::new(None);

pub fn set_activesong(title: Option<String>) -> bool {
    let mut cur = ACTIVESONG.lock().unwrap();
    if *cur == title {
        return false;
    }
    *cur = title;
    true
}

pub fn is_unset(key: &str, value: &VarValue) -> bool {
    key == "iwwc.activesong" && matches!(value, VarValue::Str(s) if s.is_empty())
}

#[cfg(test)]
pub fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn values() -> Vec<(String, VarValue)> {
    let mut out = Vec::new();
    out.push((
        "iwwc.activesong".to_string(),
        VarValue::Str(ACTIVESONG.lock().unwrap().clone().unwrap_or_default()),
    ));
    if let Ok(text) = std::fs::read_to_string("/proc/meminfo")
        && let Some((total, used)) = read_meminfo(&text)
    {
        out.push(("iwwc.ram.total".to_string(), VarValue::Int(total)));
        out.push(("iwwc.ram.used".to_string(), VarValue::Int(used)));
    }
    let mut sys = SYS
        .get_or_init(|| Mutex::new(System::new()))
        .lock()
        .unwrap();
    sys.refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage().with_frequency());
    for (i, cpu) in sys.cpus().iter().enumerate() {
        out.push((
            format!("iwwc.cpu.{i}.usage"),
            VarValue::Float(round2(cpu.cpu_usage())),
        ));
        out.push((
            format!("iwwc.cpu.{i}.frequency"),
            VarValue::Int(cpu.frequency() as i128),
        ));
    }
    out.push((
        "iwwc.cpu.avg.usage".to_string(),
        VarValue::Float(round2(sys.global_cpu_usage())),
    ));
    out
}

fn round2(v: f32) -> f64 {
    (v as f64 * 100.0).round() / 100.0
}

pub fn children(values: &[(String, VarValue)], prefix: &str) -> Vec<String> {
    let dotted = format!("{prefix}.");
    let mut out: Vec<String> = Vec::new();
    for (key, _) in values {
        if let Some(rest) = key.strip_prefix(&dotted) {
            let seg = rest.split('.').next().unwrap_or(rest);
            if !out.iter().any(|s| s == seg) {
                out.push(seg.to_string());
            }
        }
    }
    out
}

pub fn namespace_of(name: &str) -> Option<&'static str> {
    if name == "iwwc.activesong" {
        return Some("iwwc.activesong");
    }
    ["iwwc.ram", "iwwc.cpu"].into_iter().find(|ns| {
        name.strip_prefix(ns)
            .is_some_and(|rest| rest.starts_with('.'))
    })
}

pub fn poll_interval(namespace: &str) -> Option<Duration> {
    match namespace {
        "iwwc.ram" | "iwwc.cpu" => Some(Duration::from_secs(1)),
        _ => None,
    }
}

fn read_meminfo(text: &str) -> Option<(i128, i128)> {
    let mut total_kb: Option<i128> = None;
    let mut avail_kb: Option<i128> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total_kb = parse_kb(rest);
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            avail_kb = parse_kb(rest);
        }
    }
    let total = total_kb?;
    let avail = avail_kb?;
    let used = (total - avail).max(0);
    Some((total * 1024, used * 1024))
}

fn parse_kb(s: &str) -> Option<i128> {
    s.split_whitespace().next()?.parse::<i128>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meminfo_parsed_to_bytes() {
        let sample = "MemTotal:       8000 kB\nMemFree: 1000 kB\nMemAvailable:   2000 kB\n";
        assert_eq!(read_meminfo(sample), Some((8000 * 1024, 6000 * 1024)));
    }

    #[test]
    fn meminfo_missing_fields_none() {
        assert_eq!(read_meminfo("MemFree: 1000 kB\n"), None);
    }

    #[test]
    fn namespace_and_interval() {
        assert_eq!(namespace_of("iwwc.ram.used"), Some("iwwc.ram"));
        assert_eq!(namespace_of("iwwc.ram"), None);
        assert_eq!(namespace_of("x"), None);
        assert_eq!(poll_interval("iwwc.ram"), Some(Duration::from_secs(1)));
        assert_eq!(poll_interval("iwwc.disk"), None);
        assert_eq!(namespace_of("iwwc.activesong"), Some("iwwc.activesong"));
        assert_eq!(poll_interval("iwwc.activesong"), None);
    }
}
