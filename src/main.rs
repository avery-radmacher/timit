mod cli;
mod core;

use std::io::{self, BufWriter};

fn main() -> io::Result<()> {
    cli::run(&mut BufWriter::new(io::stdout()))
}
