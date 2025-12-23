use std::io;

use crate::command;
use crate::input::{self, Input};

pub fn start_repl<R: io::BufRead, W: io::Write>(reader: &mut R, writer: &mut W) {
    // Init
    print!("$ ");
    writer.flush().unwrap();
    let mut buf = String::new();

    // Read
    while reader.read_line(&mut buf).is_ok() {
        let input = input::parse_input(buf.trim());
        let (cmd, args) = (&input[0], input[1..].to_vec());

        // Eval
        let output = eval(cmd, args.to_vec());

        // Print
        _ = writer.write(&output);

        // Restart
        buf.clear();
        print!("$ ");
        writer.flush().unwrap();
    }
}

fn eval(cmd: &str, args: Input) -> command::Output {
    match command::run(cmd, args) {
        Ok(output) => output,
        Err(e) => e.to_string().into(),
    }
}
