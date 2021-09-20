mod cli;
mod core;

fn main() {
    let args = std::env::args().skip(1).collect();
    match cli::parse_args(args) {
        Err(msg) => println!("Error: {}", msg),
        Ok(args) => cli::run(args),
    };
}
