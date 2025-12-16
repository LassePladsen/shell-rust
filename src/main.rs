#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Init
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    let mut cmd: &str;

    // REPL
    while let Ok(_) = io::stdin().read_line(&mut input) {
        // Read
        cmd = input.trim();

        // Eval
        match cmd {
            "exit" => std::process::exit(0),
            _ => println!("{cmd}: command not found"), // invalid input
        }

        // Restart
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
    }
}
