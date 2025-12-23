use std::io;

mod command;
mod env;
mod file;
mod input;
mod repl;

fn main() {
    repl::start_repl(&mut io::BufReader::new(io::stdin()), &mut io::stdout());
}
