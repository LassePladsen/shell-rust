use std::io::{self, Write};

use crate::command;
use crate::input::{self, Input};

pub fn start_repl<R: io::BufRead>(reader: &mut R) {
    // Init
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();

    // Read
    while reader.read_line(&mut buf).is_ok() {
        let input = input::parse_input(buf.trim());
        let (cmd, args) = (&input[0], input[1..].to_vec());

        // Eval
        let output = eval(cmd, args.to_vec());

        // Print
        print!("{output}");

        // Restart
        buf.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
    }
}

fn eval(cmd: &str, args: Input) -> String {
    match command::run(cmd, args) {
        Ok(output) => str::from_utf8(&output)
            .expect("Could not write bytes to string")
            .to_string(),
        Err(e) => e.to_string(),
    }
}
