use std::io;

mod cmd;
mod repl;

fn main() {
    repl::start_repl(&mut io::BufReader::new(io::stdin()));
}
