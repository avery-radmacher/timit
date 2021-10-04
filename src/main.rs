mod cli;
mod core;

use std::io;

fn main() -> io::Result<()> {
    cli::run(&mut io::BufWriter::new(io::stdout()))
}
