use std::collections::HashMap;

use sysinfo::Pid;

use crate::process::PortInfo;

pub fn get_pid_port_table() -> HashMap<Pid, Vec<PortInfo>> {
    // TODO: Get process listening ports for Windows use Winapi.
    HashMap::new()
}
