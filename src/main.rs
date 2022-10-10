use std::collections::HashMap;

use inquire::{error::InquireError, Select};
use once_cell::sync::Lazy;
use regex::Regex;
use std::process::Command;
use sysinfo::{Process, System, SystemExt};
use unicode_width::UnicodeWidthStr;

static REGEX_WHITE_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
static SYSTEM: Lazy<System> = Lazy::new(|| {
    let mut sys = System::new();
    sys.refresh_all();
    sys
});

struct PortInfo {
    protocol: String,
    port: u16,
}

impl std::fmt::Display for PortInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.protocol)?;
        f.write_str(":")?;
        f.write_fmt(format_args!("{}", self.port))
    }
}

struct ProcessInfo<'a> {
    process: &'a Process,
    ports: Vec<PortInfo>,
}

impl<'a> std::fmt::Display for ProcessInfo<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix_space_count = 35usize.saturating_sub(self.process.name.width_cjk());
        let prefix = " ".repeat(prefix_space_count);

        f.write_fmt(format_args!(
            "{:>8} | {prefix}{} | CPU {:>5.2} | MEM {:>5.0}MB | ",
            self.process.pid,
            self.process.name,
            self.process.cpu_usage,
            self.process.memory / 1024,
        ))?;

        for port in &self.ports {
            f.write_fmt(format_args!("{} ", port))?;
        }

        Ok(())
    }
}

fn try_parse_lsof_line(line: &str) -> Option<(i32, PortInfo)> {
    let parts: Vec<_> = REGEX_WHITE_SPACES.split(line).collect();
    let pid = parts.get(1)?.parse().ok()?;
    let protocol = parts.get(7)?.to_string();
    let port = parts.get(8)?.split(':').nth(1)?.parse().ok()?;
    let port_info = PortInfo { protocol, port };
    Some((pid, port_info))
}

fn get_pid_port_table() -> HashMap<i32, Vec<PortInfo>> {
    let output = Command::new("lsof")
        .args(&["-nP"])
        .output()
        .expect("Failed to execute lsof");

    let output = String::from_utf8_lossy(&output.stdout);
    let mut table: HashMap<i32, Vec<PortInfo>> = HashMap::new();

    output
        .lines()
        .filter(|line| line.contains("LISTEN"))
        .flat_map(try_parse_lsof_line)
        .for_each(|(pid, port)| {
            table.entry(pid).or_default().push(port);
        });

    table
}

fn kill_process_by_pid(pid: i32) {
    Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .spawn()
        .unwrap();
}

struct App {}

impl App {
    fn new() -> Self {
        Self {}
    }

    fn get_options(&mut self) -> Vec<ProcessInfo<'static>> {
        let list = SYSTEM.get_process_list();
        let mut table = get_pid_port_table();

        let mut process_list = list.values().collect::<Vec<&Process>>();
        process_list.sort_by(|a, b| a.cpu_usage.total_cmp(&b.cpu_usage));

        process_list
            .into_iter()
            .map(|process| {
                let ports = table.remove(&process.pid).unwrap_or_default();
                ProcessInfo { process, ports }
            })
            .collect()
    }

    pub fn start(&mut self) {
        let options = self.get_options();
        let ans: Result<ProcessInfo<'static>, InquireError> =
            Select::new("Select process to kill:", options).prompt();
        match ans {
            Ok(choice) => {
                kill_process_by_pid(choice.process.pid);
                println!(
                    "Process {}({}) killed.",
                    choice.process.name, choice.process.pid
                )
            }
            Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => (),
            _ => println!("There was an error, please try again"),
        }
    }
}

fn main() {
    let mut app = App::new();
    app.start();
}
