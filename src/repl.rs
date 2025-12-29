use std::io::{BufRead, Write};

use crate::command;
use crate::input::{self, CommandParams, Input};

pub fn start_repl<R: BufRead, W: Write>(reader: &mut R, stdwriter: &mut W) {
    // Init
    print!("$ ");
    stdwriter.flush().unwrap();
    let mut buf = String::new();

    // Read
    while reader.read_line(&mut buf).is_ok() {
        let params = input::parse_input(buf.trim());
        let (cmd, args) = (&params.input[0], &params.input[1..]);

        // Eval
        let output = eval(cmd, args.to_vec());

        // Print
        _ = stdwriter.write(&output);

        // Restart
        buf.clear();
        print!("$ ");
        stdwriter.flush().unwrap();
    }
}

fn eval(cmd: &str, args: Input) -> command::Output {
    match command::run(cmd, args) {
        Ok(output) => output,
        Err(e) => e.to_string().into(),
    }
}
