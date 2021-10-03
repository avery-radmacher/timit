mod cli;
mod core;

use std::io;

fn main() {
    cli::run(&mut io::BufWriter::new(io::stdout()));
}
