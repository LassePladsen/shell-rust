use std::io;

mod command;
mod repl;
mod env;
mod file;
mod args;

fn main() {
    repl::start_repl(&mut io::BufReader::new(io::stdin()));
}
