use std::io::{self, Write};

use crate::cmd;

pub fn start_repl() {
    // Init
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    let mut cmd: &str;

    // REPL
    while io::stdin().read_line(&mut buf).is_ok() {
        // Read
        cmd = buf.trim();

        eval(cmd);

        // Restart
        buf.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
    }
}

pub fn eval(input: &str) {
    let mut words = input.split_whitespace();
    let cmd = words.next().expect("Could not find command in the input: '{input}'");
    let args: Vec<&str> = words.collect();

    match cmd {
        "exit" => cmd::exit(),
        "echo" => cmd::echo(args),
        _ => cmd::notfound(cmd), // invalid input
    }
}
