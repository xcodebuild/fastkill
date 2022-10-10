use inquire::{InquireError, Select};
use sysinfo::ProcessExt;

use crate::process::ProcessInfo;

mod process;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

struct App {}

impl App {
    fn new() -> Self {
        Self {}
    }

    pub fn start(&mut self) {
        let options = process::get_process_list();
        let ans: Result<ProcessInfo<'static>, InquireError> =
            Select::new("Select process to kill:", options).prompt();
        match ans {
            Ok(choice) => {
                choice.process.kill();
                println!(
                    "Process {}({}) killed.",
                    choice.process.name(),
                    choice.process.pid()
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
