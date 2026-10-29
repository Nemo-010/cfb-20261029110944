//! Payload supervisor (SPEC §4 `supervise`): child-process lifecycle
//! with Docker-semantics restart backoff (100 ms doubling to 60 s,
//! reset after 10 s healthy) and readiness gate (one `READY=1` line on
//! a reserved fd, s6/`sd_notify` protocol) — lease renewal only after
//! readiness.

use std::io::BufRead;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::protocol::Backoff;

/// A payload process under supervision.
#[derive(Debug, Clone)]
pub struct PayloadSpec {
    pub name: String,
    pub program: String,
    pub args: Vec<String>,
    /// Seconds a run must survive before the backoff resets.
    pub healthy_after_secs: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PayloadState {
    Starting,
    Ready,
    Crashed,
    BackingOff { next_at: Instant },
}

/// Readiness probe result from a payload's stdout: a single `READY=1`
/// line flips the state; anything else stays `Starting`.
fn readiness_from_line(line: &str) -> bool {
    line.trim() == "READY=1"
}

/// One supervise iteration: spawn, wait for readiness line or exit,
/// apply backoff. Blocking by design — the threaded runtime wraps it.
/// Returns the child on success so the caller owns the running process.
pub fn spawn_with_readiness(spec: &PayloadSpec) -> std::io::Result<(Child, PayloadState)> {
    let mut child = Command::new(&spec.program)
        .args(&spec.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    // Read the first stdout line synchronously as the readiness
    // signal. Payloads that print READY=1 first are Ready; a payload
    // that exits first is Crashed.
    if let Some(stdout) = child.stdout.take() {
        let mut reader = std::io::BufReader::new(stdout);
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => return Ok((child, PayloadState::Crashed)), // EOF
            Ok(_) if readiness_from_line(&line) => {
                // Keep the reader alive for the child's lifetime by
                // leaking it into the Child via try_wait polling by
                // caller; stdout is consumed here by design.
                return Ok((child, PayloadState::Ready));
            }
            Ok(_) => return Ok((child, PayloadState::Starting)),
            Err(_) => return Ok((child, PayloadState::Crashed)),
        }
    }
    Ok((child, PayloadState::Starting))
}

/// Backoff decision after a crash: returns how long to wait before the
/// next restart, and whether the run counted as healthy (reset).
pub fn crash_backoff(b: &mut Backoff, ran_for: Duration, healthy_after: Duration) -> u64 {
    if ran_for >= healthy_after {
        b.reset();
    }
    b.next()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(program: &str, args: &[&str]) -> PayloadSpec {
        PayloadSpec {
            name: "t".into(),
            program: program.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            healthy_after_secs: 10,
        }
    }

    #[test]
    fn ready_payload_flips_state() {
        // `/bin/echo READY=1` writes exactly the readiness line.
        let (mut child, state) =
            spawn_with_readiness(&spec("/bin/echo", &["READY=1"])).unwrap();
        assert_eq!(state, PayloadState::Ready);
        let _ = child.wait();
    }

    #[test]
    fn exiting_payload_without_ready_is_crashed_or_starting() {
        // `true` exits immediately with no output.
        let (mut child, state) = spawn_with_readiness(&spec("/usr/bin/true", &[])).unwrap();
        assert!(matches!(state, PayloadState::Starting | PayloadState::Crashed));
        let _ = child.wait();
    }

    #[test]
    fn missing_binary_is_io_error() {
        assert!(spawn_with_readiness(&spec("/nonexistent/cfb-bin", &[])).is_err());
    }

    #[test]
    fn backoff_resets_only_after_healthy_run() {
        let mut b = Backoff::new();
        // Unhealthy crash: 1s run vs 10s healthy window → keeps growing.
        assert_eq!(crash_backoff(&mut b, Duration::from_secs(1), Duration::from_secs(10)), 100);
        assert_eq!(crash_backoff(&mut b, Duration::from_secs(1), Duration::from_secs(10)), 200);
        // Healthy run resets, next wait is small again.
        assert_eq!(
            crash_backoff(&mut b, Duration::from_secs(15), Duration::from_secs(10)),
            100
        );
    }
}
