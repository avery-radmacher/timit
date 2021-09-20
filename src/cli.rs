use crate::core::{self, types::*};
use std::process::ExitStatus;
use std::time::Duration;

pub fn parse_args(args: Vec<String>) -> MsgResult<'static, Args> {
    let mut iter = args.into_iter();
    let mut display_nanos = false;
    let mut borrow_stdio = true;
    let mut command = None;
    loop {
        let arg = match iter.next() {
            None => break,
            Some(arg) => arg,
        };
        if arg == String::from("--nanos") {
            display_nanos = true;
        } else if arg == String::from("--hide-stdio") {
            borrow_stdio = false;
        } else if arg == String::from("--prog") {
            command = iter.next();
            break;
        } else {
            command = Some(arg);
            break;
        }
    }
    let command_args = iter.collect();
    match command {
        None => Err("No command specified"),
        Some(command) => Ok(Args {
            display_nanos,
            borrow_stdio,
            command,
            command_args,
        }),
    }
}

pub fn run(args: Args) {
    initialize(&args);
    println!("-- Begin program output --");
    let results = core::observe_process(&args);
    println!("--- End program output ---");
    match results {
        Ok(results) => print_results(&args, results),
        Err(reason) => println!("Error: {}", reason),
    }
}

fn initialize(args: &Args) {
    let mut command = format!("timit {}", args.command);
    for arg in &args.command_args {
        command.push_str(&format!(" {}", arg));
    }
    println!("Command: {}", command);
}

fn print_results(args: &Args, results: ProcessResults) {
    println!("Results:");
    println!(
        "  Exit status: {}",
        exit_status_to_string(results.exit_status)
    );
    println!(
        "  Duration: {}",
        duration_to_string(results.duration, !args.display_nanos)
    );
    println!();
}

fn exit_status_to_string(status: ExitStatus) -> String {
    let code = match status.code() {
        Some(code) => format!("{}", code),
        None => String::from("signal"),
    };
    let explanation = if status.success() {
        "success"
    } else {
        "failure"
    };
    format!("{} ({})", code, explanation)
}

fn duration_to_string(duration: MsgResult<Duration>, pretty: bool) -> String {
    if let Err(msg) = duration {
        return String::from(msg);
    }
    let total_nanos = duration.unwrap().as_nanos();
    if pretty {
        let hours = if total_nanos >= 3_600_000_000_000 {
            format!("{:02}:", total_nanos / 3_600_000_000_000)
        } else {
            String::new()
        };
        let minutes = if total_nanos >= 60_000_000_000 {
            format!("{:02}:", total_nanos / 60_000_000_000 % 60)
        } else {
            String::new()
        };
        let seconds = if total_nanos >= 1_000_000_000 {
            format!("{:02}", total_nanos / 1_000_000_000 % 60)
        } else {
            String::from("0")
        };
        let nanos = format!("{:09}", total_nanos % 1_000_000_000);
        format!("{}{}{}.{}s", hours, minutes, seconds, nanos)
    } else {
        format!("{}ns", total_nanos)
    }
}
