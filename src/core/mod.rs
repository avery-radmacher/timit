pub mod types;
use std::process::{Command, Stdio};
use std::time::Instant;
use types::*;

pub fn observe_process(args: &Args, io: IOArgs) -> Result<ProcessData> {
    let mut command = build_command(args, io);
    let start_time = Instant::now();
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => return Err(Error::NotSpawned),
    };
    let exit_status = match child.wait() {
        Ok(status) => status,
        Err(_) => return Err(Error::NotJoined),
    };
    let end_time = Instant::now();

    Ok(ProcessData {
        exit_status,
        duration: end_time.checked_duration_since(start_time),
    })
}

fn build_command(args: &Args, io: IOArgs) -> Command {
    let mut command = Command::new(&args.command);
    command
        .args(&args.command_args)
        .stdout(stream_or_null(io.stdout))
        .stderr(stream_or_null(io.stderr))
        .stdin(stream_or_null(io.stdin));
    command
}

fn stream_or_null(file: Option<std::fs::File>) -> Stdio {
    match file {
        Some(file) => Stdio::from(file),
        None => Stdio::inherit(),
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            Error::NotSpawned => String::from("could not spawn process"),
            Error::NotJoined => String::from("could not observe process exit"),
        };
        write!(f, "{}", reason)
    }
}
