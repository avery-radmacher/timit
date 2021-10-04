use crate::core::{self, types::*};
use std::io::{self, Write};
use std::process::ExitStatus;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "timit",
    about = "A simple program execution reporter",
    setting = structopt::clap::AppSettings::TrailingVarArg,
)]
struct CLIArgs {
    /// Display execution time as integer nanos instead of prettified
    #[structopt(short, long)]
    nanos: bool,

    /// Don't share terminal stdio with spawned process
    #[structopt(short, long)]
    hide_stdio: bool,

    /// The command to spawn followed by its arguments
    #[structopt(required = true)]
    command: Vec<String>,
}

impl CLIArgs {
    pub fn to_args(self) -> Args {
        let mut command_iter = self.command.into_iter();
        Args {
            display_nanos: self.nanos,
            borrow_stdio: !self.hide_stdio,
            command: command_iter.next().unwrap(), // failsafe due to #[structopt(required = true)]
            command_args: command_iter.collect(),
        }
    }
}

pub fn run(writer: &mut impl Write) -> io::Result<()> {
    let args = CLIArgs::from_args().to_args();
    initialize(&args, writer)?;
    writeln!(writer, "-- Begin program output --")?;
    writer.flush()?;
    let results = core::observe_process(&args);
    writeln!(writer, "--- End program output ---")?;
    match results {
        Ok(results) => print_results(&args, results, writer),
        Err(reason) => writeln!(writer, "Error: {}", reason),
    }
}

fn initialize(args: &Args, writer: &mut impl Write) -> io::Result<()> {
    let mut command = format!("timit {}", args.command);
    for arg in &args.command_args {
        command.push_str(&format!(" {}", arg));
    }
    writeln!(writer, "Command: {}", command)
}

fn print_results(args: &Args, results: ProcessResults, writer: &mut impl Write) -> io::Result<()> {
    writeln!(writer, "Results:")?;
    writeln!(
        writer,
        "  Exit status: {}",
        exit_status_to_string(results.exit_status)
    )?;
    writeln!(
        writer,
        "  Duration: {}",
        duration_to_string(results.duration, !args.display_nanos)
    )?;
    writeln!(writer)
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
