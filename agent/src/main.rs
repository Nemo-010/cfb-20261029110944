//! cfb-agent: the one binary per payload host (SPEC §4).
//!
//! Subcommands:
//!   cfb-agent run      — main loop: enroll (if needed) → broker → supervise
//!   cfb-agent enroll   — one-time enrollment against an offer URL
//!   cfb-agent doctor   — print the host capability table
//!
//! Scaffold: config, protocol, doctor and supervisor semantics are
//! implemented and tested; network I/O (broker WS, tunnel management)
//! is stubbed at the module boundaries and lands with real transport.

mod config;
mod doctor;
mod protocol;
mod supervise;

use std::process::exit;

const AGENT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_else(|| usage());
    match arg.as_str() {
        "run" => run(),
        "enroll" => {
            eprintln!("enroll: not implemented in scaffold (SPEC §4/enroll)");
            exit(2);
        }
        "doctor" => doctor(),
        "--version" | "version" => println!("cfb-agent {AGENT_VERSION}"),
        other => {
            eprintln!("unknown subcommand: {other}");
            usage();
        }
    }
}

fn usage() -> ! {
    eprintln!("usage: cfb-agent <run|enroll|doctor|--version>");
    exit(64);
}

fn doctor() {
    let caps = doctor::HostCapabilities::probe(AGENT_VERSION);
    println!("{}", caps.summary());
}

fn run() {
    let cfg = config::Config::default()
        .apply_env(|k| std::env::var(k).ok());
    if let Err(e) = cfg.validate() {
        eprintln!("config invalid: {e}");
        exit(78); // EX_CONFIG
    }
    let caps = doctor::HostCapabilities::probe(AGENT_VERSION);
    eprintln!(
        "cfb-agent {} starting: host={} broker={} caps[{}]",
        AGENT_VERSION,
        cfg.host_id,
        cfg.broker_url,
        caps.summary()
    );
    eprintln!("run loop: broker transport not implemented in scaffold");
    exit(69); // EX_UNAVAILABLE
}
