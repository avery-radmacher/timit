use std::fs::File;
use std::process::ExitStatus;
use std::time::Duration;

pub struct Args {
    pub display_nanos: bool,
    /// Ignored for now
    pub borrow_stdio: bool,
    pub command: String,
    pub command_args: Vec<String>,
}

pub struct IOArgs {
    pub stdin: Option<File>,
    pub stdout: Option<File>,
    pub stderr: Option<File>,
}

pub struct ProcessData {
    pub exit_status: ExitStatus,
    pub duration: Option<Duration>,
}

pub enum Error {
    NotSpawned,
    NotJoined,
}

pub type Result<T> = std::result::Result<T, Error>;
