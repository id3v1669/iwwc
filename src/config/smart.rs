use crate::config::types::VarValue;
use std::time::Duration;

pub fn values() -> Vec<(String, VarValue)> {
    let mut out = Vec::new();
    if let Ok(text) = std::fs::read_to_string("/proc/meminfo")
        && let Some((total, used)) = read_meminfo(&text)
    {
        out.push(("iwwc.ram.total".to_string(), VarValue::Int(total)));
        out.push(("iwwc.ram.used".to_string(), VarValue::Int(used)));
    }
    out
}

pub fn namespace_of(name: &str) -> Option<&'static str> {
    match name {
        "iwwc.ram.total" | "iwwc.ram.used" => Some("iwwc.ram"),
        _ => None,
    }
}

pub fn poll_interval(namespace: &str) -> Option<Duration> {
    match namespace {
        "iwwc.ram" => Some(Duration::from_secs(1)),
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
    }
}
