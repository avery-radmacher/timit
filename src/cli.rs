use crate::core::{self, types::*};
use std::process::ExitStatus;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "timit", about = "A simple program execution reporter")]
struct CLIArgs {
    /// Display execution time as integer nanos instead of prettified
    #[structopt(short, long)]
    pub nanos: bool,

    /// Don't share terminal stdio with spawned process
    #[structopt(short, long)]
    pub hide_stdio: bool,

    /// The command to spawn
    pub command: String,

    /// The arguments to pass the command
    pub args: Vec<String>,
}

impl CLIArgs {
    pub fn to_args(self) -> Args {
        Args {
            display_nanos: self.nanos,
            borrow_stdio: !self.hide_stdio,
            command: self.command,
            command_args: self.args,
        }
    }
}

pub fn run() {
    let args = CLIArgs::from_args().to_args();
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
