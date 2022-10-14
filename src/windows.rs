use std::{alloc::Layout, collections::HashMap};

use sysinfo::Pid;

use crate::process::PortInfo;

trait Table<T> {
    fn table(&self) -> *const T;
    fn row_count(&self) -> usize;
}

trait Row {
    fn pid(&self) -> u32;
    fn state(&self) -> u32;
    fn port(&self) -> u32;
}

trait Protocol<T: Row>: Table<T> + Default {
    fn name() -> &'static str;
    fn win32_api(table: *mut Self, size: *mut u32) -> u32;
    fn listening_ports() -> HashMap<Pid, Vec<PortInfo>> {
        let mut table = Self::default();
        let mut table_ptr = &mut table as *mut Self;
        let mut size: winapi::ctypes::c_ulong = 0;
        let size_ptr = &mut size as winapi::shared::minwindef::PULONG;
        let code = Self::win32_api(table_ptr, size_ptr);

        let buff;
        if code == winapi::shared::winerror::ERROR_INSUFFICIENT_BUFFER {
            buff =
                unsafe { std::alloc::alloc(Layout::from_size_align_unchecked(size as usize, 1)) };
            if buff.is_null() {
                panic!("get listening port list size of protocol {} failed: memory not enough, {} bytes needed", Self::name(), size);
            }
            table_ptr = buff as *mut Self;
        } else {
            panic!(
                "get listening port list size of protocol {} failed: {}",
                Self::name(),
                code
            );
        }

        let code = Self::win32_api(table_ptr, size_ptr);

        if code != winapi::shared::winerror::NO_ERROR {
            panic!(
                "get listening port list of protocol {} failed: {}",
                Self::name(),
                code
            );
        }

        let table: &mut Self = unsafe { &mut *table_ptr };
        let rows = unsafe { std::slice::from_raw_parts(table.table(), table.row_count()) };

        let mut result: HashMap<Pid, Vec<PortInfo>> = HashMap::new();

        rows.iter()
            .filter(|row| row.state() == winapi::shared::tcpmib::MIB_TCP_STATE_LISTEN)
            .for_each(|row| {
                result
                    .entry(Pid::from(row.pid() as usize))
                    .or_default()
                    .push(PortInfo {
                        protocol: Self::name().to_string(),
                        port: unsafe {
                            winapi::um::winsock2::ntohs(row.port() as winapi::um::winsock2::u_short)
                        },
                    });
            });

        result
    }
}

macro_rules! protocol_impl {
    (stateimpl, $self:ident, dwState) => {
        $self.dwState
    };

    (stateimpl, $self:ident, State) => {
        $self.State
    };

    ($name:literal = $func:path => $t:ty { $state:tt }: [$r:ty]) => {
        impl Row for $r {
            fn pid(&self) -> u32 {
                self.dwOwningPid
            }

            fn state(&self) -> u32 {
                protocol_impl!(stateimpl, self, $state)
            }

            fn port(&self) -> u32 {
                self.dwLocalPort
            }
        }

        impl Table<$r> for $t {
            fn table(&self) -> *const $r {
                self.table.as_ptr()
            }

            fn row_count(&self) -> usize {
                self.dwNumEntries as usize
            }
        }
        impl Protocol<$r> for $t {
            fn name() -> &'static str {
                $name
            }

            fn win32_api(table: *mut Self, size: *mut u32) -> u32 {
                unsafe { $func(table, size, 0) }
            }
        }
    };
}

protocol_impl!("TCP" = winapi::um::iphlpapi::GetTcpTable2 => winapi::shared::tcpmib::MIB_TCPTABLE2 { dwState }: [winapi::shared::tcpmib::MIB_TCPROW2]);
protocol_impl!("TCP6" = winapi::um::iphlpapi::GetTcp6Table2 => winapi::shared::tcpmib::MIB_TCP6TABLE2 { State }: [winapi::shared::tcpmib::MIB_TCP6ROW2]);

// TODO: UDP?

pub fn get_pid_port_table() -> HashMap<Pid, Vec<PortInfo>> {
    let mut result: HashMap<Pid, Vec<PortInfo>> = HashMap::new();

    std::iter::empty()
        .chain(winapi::shared::tcpmib::MIB_TCPTABLE2::listening_ports())
        .chain(winapi::shared::tcpmib::MIB_TCP6TABLE2::listening_ports())
        .for_each(|(pid, ports)| {
            result.entry(pid).or_default().extend(ports.into_iter());
        });

    result
}
