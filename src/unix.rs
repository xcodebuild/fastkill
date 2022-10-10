use std::collections::HashMap;
use std::process::Command;

use once_cell::sync::Lazy;
use regex::Regex;
use sysinfo::Pid;

use crate::process::PortInfo;

static REGEX_WHITE_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

fn try_parse_lsof_line(line: &str) -> Option<(Pid, PortInfo)> {
    let parts: Vec<_> = REGEX_WHITE_SPACES.split(line).collect();
    let pid = parts.get(1)?.parse().ok()?;
    let protocol = parts.get(7)?.to_string();
    let port = parts.get(8)?.split(':').nth(1)?.parse().ok()?;
    let port_info = PortInfo { protocol, port };
    Some((pid, port_info))
}

pub fn get_pid_port_table() -> HashMap<Pid, Vec<PortInfo>> {
    let output = Command::new("lsof")
        .args(&["-nP"])
        .output()
        .expect("Failed to execute lsof");

    let output = String::from_utf8_lossy(&output.stdout);
    let mut table: HashMap<Pid, Vec<PortInfo>> = HashMap::new();

    output
        .lines()
        .filter(|line| line.contains("LISTEN"))
        .filter_map(try_parse_lsof_line)
        .for_each(|(pid, port)| {
            table.entry(pid).or_default().push(port);
        });

    table
}
