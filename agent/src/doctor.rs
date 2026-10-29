//! Capability probe (SPEC §4 `doctor`): measured at enroll and on
//! demand. Values are read from the real host where the interface
//! exists; unavailable probes report `unknown` rather than guessing.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probe {
    Yes,
    No,
    Unknown,
}

impl fmt::Display for Probe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Probe::Yes => write!(f, "yes"),
            Probe::No => write!(f, "no"),
            Probe::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostCapabilities {
    pub kvm: Probe,
    pub egress_443: Probe,
    pub egress_7844: Probe,
    pub cpu_cores: Option<u32>,
    pub mem_total_mb: Option<u64>,
    pub agent_version: String,
}

impl HostCapabilities {
    pub fn probe(agent_version: &str) -> Self {
        HostCapabilities {
            kvm: probe_kvm(),
            egress_443: Probe::Unknown, // needs runtime connect test
            egress_7844: Probe::Unknown,
            cpu_cores: read_cpu_cores(),
            mem_total_mb: read_mem_total_mb(),
            agent_version: agent_version.to_string(),
        }
    }

    /// One-line summary for the enroll frame / SPA host card.
    pub fn summary(&self) -> String {
        format!(
            "kvm={} egress443={} egress7844={} cores={} mem_mb={} agent={}",
            self.kvm,
            self.egress_443,
            self.egress_7844,
            self.cpu_cores.map(|c| c.to_string()).unwrap_or_else(|| "?".into()),
            self.mem_total_mb.map(|m| m.to_string()).unwrap_or_else(|| "?".into()),
            self.agent_version,
        )
    }
}

/// KVM usable = /dev/kvm exists AND openable (some hosts expose the
/// node without granting the cgroup device).
fn probe_kvm() -> Probe {
    match std::fs::OpenOptions::new().read(true).write(true).open("/dev/kvm") {
        Ok(_) => Probe::Yes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Probe::No,
        Err(_) => Probe::Unknown, // exists but not openable
    }
}

fn read_cpu_cores() -> Option<u32> {
    std::thread::available_parallelism().ok().map(|n| n.get() as u32)
}

fn read_mem_total_mb() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/meminfo").ok()?;
    let line = s.lines().find(|l| l.starts_with("MemTotal:"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / 1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_returns_all_fields_populated() {
        let c = HostCapabilities::probe("test");
        // /proc/meminfo and available_parallelism exist on any Linux.
        assert!(c.cpu_cores.unwrap_or(0) > 0, "cpu cores must be read");
        assert!(c.mem_total_mb.unwrap_or(0) > 0, "meminfo must be read");
        assert_eq!(c.agent_version, "test");
        assert!(matches!(c.kvm, Probe::Yes | Probe::No | Probe::Unknown));
    }

    #[test]
    fn summary_is_single_line() {
        let c = HostCapabilities::probe("test");
        let s = c.summary();
        assert!(!s.contains('\n'));
        assert!(s.starts_with("kvm="));
        assert!(s.ends_with("agent=test"));
    }
}
