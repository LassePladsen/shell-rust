use std::io::{BufRead, Write};

use crate::command::{self, Output, Pipeline, run};
use crate::input::{self, ArgsSlice};

pub fn start_repl<R: BufRead, W1: Write, W2: Write>(
    reader: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) {
    // Init
    _ = stdout.write(b"$ ");
    stdout.flush().unwrap();
    let mut buf = String::new();
    let mut output: Output;

    // Read
    while reader.read_line(&mut buf).is_ok() {
        match input::parse_input(buf.trim()) {
            Ok(pipeline) => {
                // Eval
                eval_pipeline(pipeline);
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

fn eval_pipeline(pipeline: Pipeline) -> Output {
    let mut output = Output::default();
    for command in pipeline.commands {
        if !output.is_empty() {
            // We will pipe the output of the previous command into the next command
        }
        if command.args.is_empty() {
            continue;
        }
        let cmd = &command.args[0];
        let args = &command.args[1..]; // NOTE: the range 1.. should not panic and instead give an empty slice as long as the args vec has length > 0.
        output = eval_cmd(cmd, args);
    }

    // Last output goes to the pipelines writers
    output
}

fn eval_cmd(cmd: &str, args: ArgsSlice) -> Output {
    match run(cmd, args) {
        Ok(output) => output,
        Err(e) => Output {
            stderr: e.to_string().as_bytes().to_vec(),
            ..Default::default()
        },
    }
}
