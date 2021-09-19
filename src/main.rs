use domain_types::*;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

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

fn initialize(args: &Args) {
    let mut command = format!("timit {}", args.command);
    for arg in &args.command_args {
        command.push_str(&format!(" {}", arg));
    }
    println!("Command: {}", command);
}

pub mod core {
    use super::*;

    fn observe_process(args: &Args) -> MsgResult<ProcessResults> {
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

    pub fn run(args: Args) {
        initialize(&args);
        println!("-- Begin program output --");
        let results = observe_process(&args);
        println!("--- End program output ---");
        match results {
            Ok(results) => print_results(&args, results),
            Err(reason) => println!("Error: {}", reason),
        }
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

fn parse_args(args: Vec<String>) -> MsgResult<'static, Args> {
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

fn main() {
    let args = std::env::args().skip(1).collect();
    match parse_args(args) {
        Err(msg) => println!("Error: {}", msg),
        Ok(args) => core::run(args),
    };
}
