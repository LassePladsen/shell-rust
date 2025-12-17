use std::io;

mod cmd;
mod repl;
mod env;
mod file;

fn main() {
    repl::start_repl(&mut io::BufReader::new(io::stdin()));
}
