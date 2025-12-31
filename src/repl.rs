use std::io::{BufRead, Write};

use crate::command;
use crate::input::{self, Args};

pub fn start_repl<R: BufRead, W1: Write, W2: Write>(
    reader: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) {
    // Init
    _ = stdout.write(b"$ ");
    stdout.flush().unwrap();
    let mut buf = String::new();
    let mut output: command::Output;

    // Read
    while reader.read_line(&mut buf).is_ok() {
        match input::parse_input(buf.trim()) {
            Ok(pipeline) => {
                // Execute
                execute_pipeline(pipeline);
            }
            Err(e) => {
                let mut s = e.to_string();
                s.push('\n');

                // Print
                _ = stderr.write(s.as_bytes());

                // Restart
                buf.clear();
                _ = stdout.write(b"$ ");
                stdout.flush().unwrap();
            }
        };
    }
}

fn execute_pipeline()

fn eval(cmd: &str, args: Args) -> command::Output {
    match command::run(cmd, args) {
        Ok(output) => output,
        Err(e) => command::Output {
            stderr: e.to_string().as_bytes().to_vec(),
            ..Default::default()
        },
    }
}
