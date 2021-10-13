use crate::core::{self, types::*};
use std::fs::File;
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

    /// Set stdin for the spawned process
    #[structopt(short = "i", long, conflicts_with = "stdin_inherit")]
    stdin: Option<String>,

    /// Set stdout for the spawned process
    #[structopt(short = "o", long, conflicts_with = "stdout_inherit")]
    stdout: Option<String>,

    /// Set stderr for the spawned process
    #[structopt(short = "e", long, conflicts_with = "stderr_inherit")]
    stderr: Option<String>,

    /// Inherit stdin from the current process
    #[structopt(long)]
    stdin_inherit: bool,

    /// Inherit stdout from the current process
    #[structopt(long)]
    stdout_inherit: bool,

    /// Inherit stderr from the current process
    #[structopt(long)]
    stderr_inherit: bool,

    /// The command to spawn followed by its arguments
    #[structopt(required = true)]
    command: Vec<String>,
}

impl CLIArgs {
    fn to_io_stream(
        inherit: bool,
        file: Option<String>,
        is_for_stdin: bool,
    ) -> io::Result<IOStream> {
        if inherit {
            Ok(IOStream::Inherit)
        } else if let Some(file) = file {
            let stream = if is_for_stdin {
                File::open(file)?
            } else {
                File::create(file)?
            };
            Ok(IOStream::File(stream))
        } else {
            Ok(IOStream::Null)
        }
    }

    pub fn to_args(self) -> io::Result<(Args, IOArgs)> {
        let stdin = CLIArgs::to_io_stream(self.stdin_inherit, self.stdin, true)?;
        let stdout = CLIArgs::to_io_stream(self.stdout_inherit, self.stdout, false)?;
        let stderr = CLIArgs::to_io_stream(self.stderr_inherit, self.stderr, false)?;
        let mut command_iter = self.command.into_iter();
        Ok((
            Args {
                display_nanos: self.nanos,
                command: command_iter.next().unwrap(), // failsafe due to #[structopt(required = true)]
                command_args: command_iter.collect(),
            },
            IOArgs {
                stdin,
                stdout,
                stderr,
            },
        ))
    }
}

pub fn run(writer: &mut impl Write) -> io::Result<()> {
    let (args, io_args) = CLIArgs::from_args().to_args()?;
    initialize(&args, writer)?;
    writeln!(writer, "-- Begin program output --")?;
    writer.flush()?;
    let results = core::observe_process(&args, io_args);
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

fn print_results(args: &Args, results: ProcessData, writer: &mut impl Write) -> io::Result<()> {
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

fn duration_to_string(duration: Option<Duration>, pretty: bool) -> String {
    if let None = duration {
        return String::from("There was an error timing the operation.");
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
