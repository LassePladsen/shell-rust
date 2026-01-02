use std::io::{BufRead, Write};

use crate::command::{Output, Pipeline, run};
use crate::input::{self, ArgsSlice};

pub fn start_repl(reader: &mut impl BufRead, stdout: &mut impl Write, stderr: &mut impl Write) {
    // Init
    _ = stdout.write(b"$ ");
    stdout.flush().unwrap();
    let mut buf = String::new();
    // Read
    loop {
        match reader.read_line(&mut buf) {
            Ok(0) => break,  // EOF reached
            Err(_) => break, // Error reading

            // Normal line
            // TODO: check EOF reached in the buffer too?
            Ok(_) => {
                match input::parse_input(buf.trim()) {
                    Ok(pipeline) => {
                        // Eval
                        let output = eval_pipeline(&pipeline);

                        // Print
                        match pipeline.stdout {
                            Some(mut writer) => {
                                writer.write_all(&output.stdout).unwrap();
                                writer.flush().unwrap();
                            }
                            None => {
                                stdout.write_all(&output.stdout).unwrap();
                                stdout.flush().unwrap();
                            }
                        }

                        match pipeline.stderr {
                            Some(mut writer) => {
                                writer.write_all(&output.stderr).unwrap();
                                writer.flush().unwrap();
                            }
                            None => {
                                stderr.write_all(&output.stderr).unwrap();
                                stderr.flush().unwrap();
                            }
                        }
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

// Claude unit tests >:)
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Somehow all output has '$' on each line even though it shouldnt be printed. My fault, it
    // doesnt happen on cargo run
    fn get_repl_output(input: &[u8]) -> (String, String) {
        let mut reader = Cursor::new(input);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        start_repl(&mut reader, &mut stdout, &mut stderr);
        (
            String::from_utf8(stdout).unwrap(),
            String::from_utf8(stderr).unwrap(),
        )
    }

    #[test]
    fn repl_single_command() {
        let (stdout, stderr) = get_repl_output(b"echo hello\n");
        assert_eq!(stdout, "$ hello\n$ ");
        assert_eq!(stderr, "");
    }

    #[test]
    fn repl_multiple_commands() {
        let (stdout, stderr) = get_repl_output(b"echo first\necho second\n");
        assert_eq!(stdout, "$ first\n$ second\n$ ");
        assert_eq!(stderr, "");
    }

    #[test]
    fn repl_empty_input() {
        let (stdout, stderr) = get_repl_output(b"\n");
        assert_eq!(stdout, "$ $ ");
        assert_eq!(stderr, "");
    }

    #[test]
    // TODO: make the program adhere to this (do error if nothing after pipe)
    fn repl_parse_error() {
        let (stdout, stderr) = get_repl_output(b"|\n");
        assert_eq!(stdout, "$ ");
        assert_eq!(stderr, "");
    }

    #[test]
    fn repl_nonexisting_cmd() {
        let (stdout, stderr) = get_repl_output(b"non_existing\n");
        assert_eq!(stdout, "$ $ ");
        assert_eq!(stderr, "non_existing: not found\n");
    }
}
