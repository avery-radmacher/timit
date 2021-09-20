mod cli;

pub mod core {
    use crate::domain_types::*;
    use std::process::{Command, Stdio};
    use std::time::Instant;

    pub fn observe_process(args: &Args) -> MsgResult<ProcessResults> {
        let mut command = Command::new(&args.command);
        command.args(&args.command_args);
        if !args.borrow_stdio {
            command
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .stdin(Stdio::null());
        }
        let start_time = Instant::now();
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(_) => return Err("Could not spawn timed process"),
        };
        let exit_status = match child.wait() {
            Ok(status) => status,
            Err(_) => return Err("Could not collect timed process exit status"),
        };
        let end_time = Instant::now();

        Ok(ProcessResults {
            exit_status,
            duration: end_time
                .checked_duration_since(start_time)
                .ok_or("There was an error timing the operation."),
        })
    }
}

pub mod domain_types {
    use std::process::ExitStatus;
    use std::time::Duration;

    pub struct Args {
        pub display_nanos: bool,
        pub borrow_stdio: bool,
        pub command: String,
        pub command_args: Vec<String>,
    }

    pub struct ProcessResults<'a> {
        pub exit_status: ExitStatus,
        pub duration: MsgResult<'a, Duration>,
    }

    pub type MsgResult<'a, T> = Result<T, &'a str>;
}

fn main() {
    let args = std::env::args().skip(1).collect();
    match cli::parse_args(args) {
        Err(msg) => println!("Error: {}", msg),
        Ok(args) => cli::run(args),
    };
}
