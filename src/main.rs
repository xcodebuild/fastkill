use std::{collections::HashMap, os::unix::process};

use inquire::{error::InquireError, Select};
use regex::Regex;
use std::process::Command;
use sysinfo::{Process, System, SystemExt};

struct App {
    system: System,
}

struct PortInfo {
    protocol: String,
    port: String,
}

fn get_pid_port_table() -> HashMap<String, PortInfo> {
    let output = Command::new("lsof")
        .args(&["-nP"])
        .output()
        .expect("Failed to execute lsof");

    let output = String::from_utf8_lossy(&output.stdout);
    let output = output.split('\n');
    let mut table = HashMap::new();
    output.into_iter().for_each(|line| {
        if line.contains("LISTEN") {
            let regex = Regex::new(r"\s+").unwrap();
            let line = regex.split(line).collect::<Vec<&str>>();
            let addressPort = line[8].split(':').collect::<Vec<&str>>();
            let pid = line[1].to_string();
            let portInfo = PortInfo {
                protocol: line[7].to_string(),
                port: addressPort[1].to_string(),
            };
            table.insert(pid, portInfo);
        }
    });
    table
}

fn kill_process_by_pid(pid: String) -> () {
    let pid = pid.parse::<i32>().unwrap();
    Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .spawn()
        .unwrap();
}

impl App {
    fn new() -> Self {
        let mut system = System::new();
        Self { system }
    }

    fn get_options(&mut self) -> Vec<String> {
        self.system.refresh_all();
        let list = self.system.get_process_list();

        let table = get_pid_port_table();

        let mut values = list.values().collect::<Vec<&Process>>();

        values.sort_by(|a, b| {
            if a.cpu_usage > b.cpu_usage {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        let options: Vec<String> = values
            .into_iter()
            .map(|p| {
                let name = &p.name;
                let pid = &p.pid.to_string();

                let port = match table.get(pid) {
                    Some(portInfo) => format!("LISTEN {}:{}", portInfo.protocol, portInfo.port),
                    None => "NOT LISTEN".to_string(),
                };
                format!(
                    "{:>8} | {:>35} | {:>10} | CPU {:.2}% | MEM {:.0}MB",
                    pid,
                    name,
                    port,
                    p.cpu_usage,
                    p.memory / 1024
                )
            })
            .collect();

        return options;
    }

    pub fn start(&mut self) {
        let options = self.get_options();
        let ans: Result<String, InquireError> =
            Select::new("Select process to kill:", options).prompt();
        match ans {
            Ok(choice) => {
                let pid = choice.split('|').collect::<Vec<&str>>()[0];
                kill_process_by_pid(pid.trim().to_string());
                println!("Process {} killed.", pid)
            }
            Err(_) => println!("There was an error, please try again"),
        }
    }
}

fn main() {
    let mut app = App::new();
    app.start();
}
