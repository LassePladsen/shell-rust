#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    let mut cmd: &str = "";
    while let Ok(_) = io::stdin().read_line(&mut input) {
        cmd = input.trim();
        // invalid input
        println!("{cmd}: command not found");
    }
}
