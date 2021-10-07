pub mod types;
use std::process::{Command, Stdio};
use std::time::Instant;
use types::*;

pub fn observe_process(args: &Args, io: IOArgs) -> MsgResult<ProcessResults> {
    let mut command = Command::new(&args.command);
    command
        .args(&args.command_args)
        .stdout(stream_or_null(io.stdout))
        .stderr(stream_or_null(io.stderr))
        .stdin(stream_or_null(io.stdin));
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

fn stream_or_null(file: Option<std::fs::File>) -> Stdio {
    match file {
        Some(file) => Stdio::from(file),
        None => Stdio::inherit(),
    }
}
