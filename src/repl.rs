use std::io::{BufRead, Write};

use crate::command::{Output, Pipeline, run};
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

    // Read
    while reader.read_line(&mut buf).is_ok() {
        match input::parse_input(buf.trim()) {
            Ok(mut pipeline) => {
                // Eval
                let output = eval_pipeline(&pipeline);

                // Print
                // Print
                _ = pipeline.stdout_writer.write(&output.stdout);
                _ = pipeline.stderr_writer.write(&output.stderr);
                pipeline.stdout_writer.flush().unwrap();
            }
            Err(e) => {
                let mut s = e.to_string();
                s.push('\n');

                // Print
                _ = stderr.write(s.as_bytes());
            }
        };
        // Restart
        buf.clear();
        _ = stdout.write(b"$ ");
        stdout.flush().unwrap();
    }
}

fn eval_pipeline(pipeline: &Pipeline) -> Output {
    // First command init
    let mut output = Output::default();
    let mut prev_output: Option<Output> = None; // aka pipe output from prev command

    for command in &*pipeline.commands {
        if command.args.is_empty() {
            continue;
        }
        let cmd = &command.args[0];
        let args = &command.args[1..];

        output = eval_cmd(cmd, args, prev_output);
        prev_output = Some(output.clone());
    }

    output
}

fn eval_cmd(cmd: &str, args: ArgsSlice, pipe_output: Option<Output>) -> Output {
    match run(cmd, args, pipe_output) {
        Ok(output) => output,
        Err(e) => Output {
            stderr: e.to_string().as_bytes().to_vec(),
            ..Default::default()
        },
    }
}
