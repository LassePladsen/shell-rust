use std::io::{BufRead, Write};

use crate::command;
use crate::input::{self, Input};

pub fn start_repl<R: BufRead, W: Write>(reader: &mut R, stdwriter: &mut W) {
    // Init
    _ = stdwriter.write(b"$ ");
    stdwriter.flush().unwrap();
    let mut buf = String::new();
    let mut output: command::Output;

    // Read
    while reader.read_line(&mut buf).is_ok() {
        match input::parse_input(buf.trim()) {
            Ok(mut params) => {
                let (cmd, args) = (&params.input[0], &params.input[1..]);

                // Eval
                output = eval(cmd, args.to_vec());

                // Print
                _ = params.out_writer.write(&output);
                params.out_writer.flush().unwrap();

                // Restart
                buf.clear();
                _ = stdwriter.write(b"$ ");
                stdwriter.flush().unwrap();
            }
            Err(e) => {
                let mut s = e.to_string();
                s.push('\n');
                output = s.into();

                // TODO: separate stdout and stderr writers?
                // Print
                _ = stdwriter.write(&output);

                // Restart
                buf.clear();
                _ = stdwriter.write(b"$ ");
                stdwriter.flush().unwrap();
            }
        };
    }
}

fn eval(cmd: &str, args: Input) -> command::Output {
    match command::run(cmd, args) {
        Ok(output) => output,
        Err(e) => e.to_string().into(),
    }
}
