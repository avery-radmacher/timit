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