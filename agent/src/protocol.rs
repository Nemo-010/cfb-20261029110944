//! Versioned wire protocol between cfb-agent and the API Worker broker.
//!
//! Normative source: SPEC §4 (`broker`) and §6.1 (mechanisms 1, 2, 7, 8).
//! Every message carries the protocol version and the cell epoch
//! (mechanism 1: epoch fencing). Enqueue dedup uses client event IDs
//! (mechanism 8: idempotent enqueue). Watchdog keepalives are in-band
//! and are the primary liveness signal (§4).

/// Wire protocol version. Bump on breaking change; negotiation on connect.
pub const PROTOCOL_VERSION: u16 = 1;

/// Cell epoch: monotonic per cell, bumped on lease takeover.
/// Superseded-epoch commands are rejected (SPEC §6.1 #1).
pub type Epoch = u64;

/// Client-supplied dedup key for at-least-once broker delivery
/// (SPEC §6.1 #8).
pub type EventId = [u8; 16];

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    /// Agent → control plane, first frame after connect.
    Hello {
        protocol: u16,
        agent_version: String,
        host_id: String,
        capabilities: Vec<String>,
    },
    /// Control plane → agent, first frame in reply. Carries the
    /// advertised lost threshold: how long the control plane waits
    /// before judging this agent's cells lost. MUST exceed the agent's
    /// max reconnect backoff (SPEC §6.1 #2).
    HelloAck {
        protocol: u16,
        lost_threshold_ms: u64,
        retry_after_ms: Option<u64>,
    },
    /// In-band watchdog keepalive (SPEC §4). Absence of watchdog pings
    /// — not socket close — drives `unknown`/`lost` transitions.
    Watchdog { epoch: Epoch },
    /// Command with fencing epoch + dedup id.
    Command {
        epoch: Epoch,
        event_id: EventId,
        name: String,
        args: Vec<(String, String)>,
    },
    /// Agent → control plane: command result.
    CommandResult {
        epoch: Epoch,
        event_id: EventId,
        ok: bool,
        detail: String,
    },
    /// Metrics frame (3 s cadence, SPEC §4 `monitor`).
    Metrics {
        epoch: Epoch,
        cpu_pct: f32,
        mem_used_mb: u64,
        load1: f32,
    },
    /// Routing-table push, rev-tagged; stale revs ignored
    /// (SPEC §6.1 #3).
    RouteTablePush { rev: u64, entries: Vec<(String, String)> },
}

impl Message {
    /// Line-based frame encoding: `TYPE|k=v;k=v;...`.
    /// Deliberately dumb; replaced by the multiplexed binary frame
    /// with version negotiation (mirrord `protocol-api` shape) once
    /// the transport is real. Kept testable here.
    pub fn encode(&self) -> String {
        let mut fields: Vec<(String, String)> = Vec::new();
        match self {
            Message::Hello { protocol, agent_version, host_id, capabilities } => {
                fields.push(("protocol".into(), protocol.to_string()));
                fields.push(("agent_version".into(), agent_version.clone()));
                fields.push(("host_id".into(), host_id.clone()));
                fields.push(("caps".into(), capabilities.join(",")));
            }
            Message::HelloAck { protocol, lost_threshold_ms, retry_after_ms } => {
                fields.push(("protocol".into(), protocol.to_string()));
                fields.push(("lost_ms".into(), lost_threshold_ms.to_string()));
                if let Some(ra) = retry_after_ms {
                    fields.push(("retry_after_ms".into(), ra.to_string()));
                }
            }
            Message::Watchdog { epoch } => {
                fields.push(("epoch".into(), epoch.to_string()));
            }
            Message::Command { epoch, event_id, name, args } => {
                fields.push(("epoch".into(), epoch.to_string()));
                fields.push(("event_id".into(), hex(event_id)));
                fields.push(("name".into(), name.clone()));
                fields.push(("args".into(), encode_pairs(args)));
            }
            Message::CommandResult { epoch, event_id, ok, detail } => {
                fields.push(("epoch".into(), epoch.to_string()));
                fields.push(("event_id".into(), hex(event_id)));
                fields.push(("ok".into(), ok.to_string()));
                fields.push(("detail".into(), detail.clone()));
            }
            Message::Metrics { epoch, cpu_pct, mem_used_mb, load1 } => {
                fields.push(("epoch".into(), epoch.to_string()));
                fields.push(("cpu".into(), format!("{cpu_pct:.2}")));
                fields.push(("mem_mb".into(), mem_used_mb.to_string()));
                fields.push(("load1".into(), format!("{load1:.2}")));
            }
            Message::RouteTablePush { rev, entries } => {
                fields.push(("rev".into(), rev.to_string()));
                fields.push(("entries".into(), encode_pairs(entries)));
            }
        }
        let type_name = match self {
            Message::Hello { .. } => "HELLO",
            Message::HelloAck { .. } => "HELLO_ACK",
            Message::Watchdog { .. } => "WATCHDOG",
            Message::Command { .. } => "CMD",
            Message::CommandResult { .. } => "CMD_RESULT",
            Message::Metrics { .. } => "METRICS",
            Message::RouteTablePush { .. } => "ROUTES",
        };
        let body: Vec<String> = fields
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        format!("{type_name}|{}", body.join(";"))
    }
}

fn encode_pairs(pairs: &[(String, String)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{k}:{v}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Reconnect backoff with doubling, 100 ms → 60 s, reset after 10 s
/// healthy (SPEC §4 `supervise`, Docker semantics).
#[derive(Debug, Clone)]
pub struct Backoff {
    next_ms: u64,
    max_ms: u64,
}

impl Backoff {
    pub fn new() -> Self {
        Backoff { next_ms: 100, max_ms: 60_000 }
    }

    pub fn next(&mut self) -> u64 {
        let v = self.next_ms;
        self.next_ms = (self.next_ms.saturating_mul(2)).min(self.max_ms);
        v
    }

    pub fn reset(&mut self) {
        self.next_ms = 100;
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new()
    }
}

/// Lost-threshold handshake check (SPEC §6.1 #2): the advertised
/// threshold must exceed the agent's max reconnect backoff, else the
/// agent refuses the session (both sides must agree on loss).
pub fn lost_threshold_ok(advertised_lost_ms: u64, max_backoff_ms: u64) -> bool {
    advertised_lost_ms > max_backoff_ms
}

/// Epoch fence check (SPEC §6.1 #1): a command/write is accepted only
/// if its epoch is not older than the currently held epoch.
pub fn epoch_current(held: Epoch, presented: Epoch) -> bool {
    presented >= held
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_roundtrip_contains_fields() {
        let m = Message::Hello {
            protocol: PROTOCOL_VERSION,
            agent_version: "0.1.0".into(),
            host_id: "host-a".into(),
            capabilities: vec!["kvm".into(), "ci".into()],
        };
        let s = m.encode();
        assert!(s.starts_with("HELLO|"));
        assert!(s.contains("protocol=1"));
        assert!(s.contains("caps=kvm,ci"));
    }

    #[test]
    fn backoff_doubles_and_caps_then_resets() {
        let mut b = Backoff::new();
        assert_eq!(b.next(), 100);
        assert_eq!(b.next(), 200);
        assert_eq!(b.next(), 400);
        b.next(); b.next(); b.next(); b.next(); b.next();
        b.next(); b.next(); b.next(); b.next(); b.next();
        assert_eq!(b.next(), 60_000, "must cap at 60s");
        b.reset();
        assert_eq!(b.next(), 100);
    }

    #[test]
    fn lost_threshold_must_exceed_max_backoff() {
        assert!(lost_threshold_ok(40_000, 30_000));
        assert!(!lost_threshold_ok(30_000, 30_000), "equal is not >");
        assert!(!lost_threshold_ok(10_000, 30_000));
    }

    #[test]
    fn superseded_epoch_rejected_current_accepted() {
        assert!(epoch_current(7, 7));
        assert!(epoch_current(7, 8), "newer epoch accepted");
        assert!(!epoch_current(7, 6), "stale epoch rejected");
    }

    #[test]
    fn command_encodes_epoch_and_event_id() {
        let m = Message::Command {
            epoch: 12,
            event_id: [0xab; 16],
            name: "restart_cell".into(),
            args: vec![("cell".into(), "counter".into())],
        };
        let s = m.encode();
        assert!(s.starts_with("CMD|"));
        assert!(s.contains("epoch=12"));
        assert!(s.contains(&hex(&[0xab; 16])));
    }
}
