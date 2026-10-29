//! Agent configuration: TOML file + `CFB_` env overrides with `__`
//! nesting (ServerBee convention, SPEC §4).
//!
//! Precedence: env > file > defaults.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub broker_url: String,
    pub host_id: String,
    pub state_dir: PathBuf,
    pub lost_threshold_ms: u64,
    pub max_backoff_ms: u64,
    pub metrics_interval_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            broker_url: "wss://cfb.example.com/api/agent/ws".into(),
            host_id: String::new(),
            state_dir: PathBuf::from("/var/lib/cfb-agent"),
            lost_threshold_ms: 40_000,
            max_backoff_ms: 30_000,
            metrics_interval_ms: 3_000,
        }
    }
}

impl Config {
    /// Parse a TOML document onto the defaults. Unknown keys are
    /// rejected so typos fail loudly.
    pub fn from_toml(s: &str) -> Result<Config, String> {
        let mut c = Config::default();
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (k, v) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid line: {line}"))?;
            let v = v.trim().trim_matches('"');
            match k.trim() {
                "broker_url" => c.broker_url = v.into(),
                "host_id" => c.host_id = v.into(),
                "state_dir" => c.state_dir = PathBuf::from(v),
                "lost_threshold_ms" => c.lost_threshold_ms = v.parse().map_err(|_| k.to_string())?,
                "max_backoff_ms" => c.max_backoff_ms = v.parse().map_err(|_| k.to_string())?,
                "metrics_interval_ms" => {
                    c.metrics_interval_ms = v.parse().map_err(|_| k.to_string())?
                }
                other => return Err(format!("unknown key: {other}")),
            }
        }
        Ok(c)
    }

    /// Apply `CFB_<KEY>` environment overrides; `__` nests (flat here:
    /// `CFB_BROKER_URL`, `CFB_HOST_ID`, ...).
    pub fn apply_env(mut self, get: impl Fn(&str) -> Option<String>) -> Self {
        if let Some(v) = get("CFB_BROKER_URL") {
            self.broker_url = v;
        }
        if let Some(v) = get("CFB_HOST_ID") {
            self.host_id = v;
        }
        if let Some(v) = get("CFB_STATE_DIR") {
            self.state_dir = PathBuf::from(v);
        }
        self
    }

    /// SPEC §6.1 #2 invariant: config as loaded must satisfy
    /// lost_threshold > max_backoff, else the agent refuses to run.
    pub fn validate(&self) -> Result<(), String> {
        if self.host_id.is_empty() {
            return Err("host_id is empty (set in TOML or CFB_HOST_ID)".into());
        }
        if self.lost_threshold_ms <= self.max_backoff_ms {
            return Err(format!(
                "lost_threshold_ms ({}) must exceed max_backoff_ms ({})",
                self.lost_threshold_ms, self.max_backoff_ms
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_overrides_defaults_and_rejects_unknown() {
        let c = Config::from_toml(
            "broker_url = \"wss://x/y\"\nhost_id = \"h1\"\n# comment\n\n",
        )
        .unwrap();
        assert_eq!(c.broker_url, "wss://x/y");
        assert_eq!(c.host_id, "h1");
        assert_eq!(c.metrics_interval_ms, 3_000, "default preserved");
        assert!(Config::from_toml("bogus = 1").is_err());
    }

    #[test]
    fn env_wins_over_file() {
        let c = Config::from_toml("host_id = \"from-file\"")
            .unwrap()
            .apply_env(|k| (k == "CFB_HOST_ID").then(|| "from-env".into()));
        assert_eq!(c.host_id, "from-env");
    }

    #[test]
    fn validate_enforces_lost_threshold_over_backoff() {
        let mut c = Config::default();
        c.host_id = "h".into();
        assert!(c.validate().is_ok(), "defaults are consistent");
        c.max_backoff_ms = c.lost_threshold_ms;
        assert!(c.validate().is_err());
        c.host_id = String::new();
        assert!(c.validate().is_err(), "empty host_id rejected");
    }
}
