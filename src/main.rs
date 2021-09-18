use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn duration_to_string(duration: Duration, pretty: bool) -> String {
    let total_nanos = duration.as_nanos();
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

fn initialize(args: &Args) {
    let mut command = format!("timit {}", args.command);
    for arg in &args.command_args {
        command.push_str(&format!(" {}", arg));
    }
    println!("Command: {}", command);
}

fn observe_process(args: &Args) -> (bool, Result<Duration, &str>) {
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
        Err(_) => return (true, Err("Could not spawn timed process")),
    };
    let success = match child.wait() {
        Ok(status) => status.success(),
        Err(_) => return (true, Err("Could not collect timed process exit status")),
    };
    let end_time = Instant::now();

    (
        success,
        end_time
            .checked_duration_since(start_time)
            .ok_or("There was an error timing the operation."),
    )
}

fn print_results(args: &Args, duration: Duration) {
    println!("Results:");
    println!(
        "  Duration: {}",
        duration_to_string(duration, !args.display_nanos)
    );
    println!();
}

fn run(args: Args) {
    initialize(&args);
    println!("-- Begin program output --");
    let (success, duration_result) = observe_process(&args);
    println!("--- End program output ---");
    if !success {
        println!("Note: Process exit status indicated failure");
    }
    match duration_result {
        Ok(duration) => print_results(&args, duration),
        Err(reason) => println!("Error: {}", reason),
    }
}

struct Args {
    display_nanos: bool,
    borrow_stdio: bool,
    command: String,
    command_args: Vec<String>,
}

fn parse_args(args: Vec<String>) -> Result<Args, &'static str> {
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
    let args: Vec<_> = std::env::args().skip(1).collect();
    match parse_args(args) {
        Err(msg) => eprintln!("Error: {}", msg),
        Ok(args) => run(args),
    };
}
