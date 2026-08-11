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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Namespace {
    Activesong,
    Ram,
    Cpu,
    Ws,
}

impl Namespace {
    pub const ALL: [Namespace; 4] = [Self::Activesong, Self::Ram, Self::Cpu, Self::Ws];

    pub fn name(self) -> &'static str {
        match self {
            Self::Activesong => "iwwc.activesong",
            Self::Ram => "iwwc.ram",
            Self::Cpu => "iwwc.cpu",
            Self::Ws => "iwwc.ws",
        }
    }

    pub fn of(var: &str) -> Option<Namespace> {
        Self::ALL.into_iter().find(|ns| match ns {
            Self::Activesong => var == ns.name(),
            _ => var
                .strip_prefix(ns.name())
                .is_some_and(|rest| rest.starts_with('.')),
        })
    }

    pub fn sample(self) -> Vec<(String, VarValue)> {
        match self {
            Self::Activesong => vec![(
                self.name().to_string(),
                VarValue::Str(ACTIVESONG.lock().unwrap().clone().unwrap_or_default()),
            )],
            Self::Ram => {
                let mut out = Vec::new();
                if let Ok(text) = std::fs::read_to_string("/proc/meminfo")
                    && let Some((total, used)) = read_meminfo(&text)
                {
                    out.push(("iwwc.ram.total".to_string(), VarValue::Int(total)));
                    out.push(("iwwc.ram.used".to_string(), VarValue::Int(used)));
                }
                out
            }
            Self::Cpu => {
                let mut out = Vec::new();
                let mut sys = SYS
                    .get_or_init(|| Mutex::new(System::new()))
                    .lock()
                    .unwrap();
                sys.refresh_cpu_specifics(
                    CpuRefreshKind::nothing().with_cpu_usage().with_frequency(),
                );
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
            Self::Ws => crate::workspace::values(),
        }
    }

    pub fn poll_interval(self) -> Option<Duration> {
        match self {
            Self::Ram | Self::Cpu => Some(Duration::from_secs(1)),
            Self::Activesong | Self::Ws => None,
        }
    }
}

pub fn values() -> Vec<(String, VarValue)> {
    Namespace::ALL
        .into_iter()
        .flat_map(Namespace::sample)
        .collect()
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
        assert_eq!(Namespace::of("iwwc.ram.used"), Some(Namespace::Ram));
        assert_eq!(Namespace::of("iwwc.ram"), None);
        assert_eq!(Namespace::of("x"), None);
        assert_eq!(Namespace::Ram.poll_interval(), Some(Duration::from_secs(1)));
        assert_eq!(
            Namespace::of("iwwc.activesong"),
            Some(Namespace::Activesong)
        );
        assert_eq!(Namespace::Activesong.poll_interval(), None);
    }
}
