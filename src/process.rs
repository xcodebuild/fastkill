use std::cmp::Reverse;

use once_cell::sync::Lazy;
use sysinfo::{Process, ProcessExt, System, SystemExt};
use unicode_width::UnicodeWidthStr;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::unix as os;
#[cfg(any(target_os = "windows"))]
use crate::windows as os;

static SYSTEM: Lazy<System> = Lazy::new(|| {
    let mut sys = System::new();
    sys.refresh_all();
    sys
});

pub struct PortInfo {
    pub protocol: String,
    pub port: u16,
}

impl std::fmt::Display for PortInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.protocol)?;
        f.write_str(":")?;
        f.write_fmt(format_args!("{}", self.port))
    }
}

pub struct ProcessInfo<'a> {
    pub process: &'a Process,
    pub ports: Vec<PortInfo>,
}

impl<'a> std::fmt::Display for ProcessInfo<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix_space_count = 35usize.saturating_sub(self.process.name().width_cjk());
        let prefix = " ".repeat(prefix_space_count);

        f.write_fmt(format_args!(
            "{:>8} | {prefix}{} | CPU {:>6.2}% | MEM {:>5.0}MB | ",
            self.process.pid().to_string(),
            self.process.name(),
            self.process.cpu_usage(),
            self.process.memory() / 1024 / 1024,
        ))?;

        for port in &self.ports {
            f.write_fmt(format_args!("{} ", port))?;
        }

        Ok(())
    }
}

pub fn get_process_list() -> Vec<ProcessInfo<'static>> {
    let list = SYSTEM.processes();
    let mut table = os::get_pid_port_table();

    let mut process_list = list.values().collect::<Vec<&Process>>();
    process_list.sort_by_key(|p| Reverse((p.cpu_usage() * 100.0) as u32));

    process_list
        .into_iter()
        .map(|process| {
            let ports = table.remove(&process.pid()).unwrap_or_default();
            ProcessInfo { process, ports }
        })
        .collect()
}
