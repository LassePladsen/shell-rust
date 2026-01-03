use std::io::{BufRead, Write};

use crate::command::{Output, Pipeline, run};
use crate::input::{self, ArgsSlice};

pub fn start_repl(reader: &mut impl BufRead, stdout: &mut impl Write, stderr: &mut impl Write) {
    // Init
    let mut buf = String::new();
    // Prompt
    _ = stdout.write(b"$ ");
    stdout.flush().unwrap();

    // Read
    loop {
        match reader.read_line(&mut buf) {
            Ok(0) => break,  // EOF reached
            Err(_) => break, // Error reading

            // Normal line
            Ok(_) => {
                // Prompt
                _ = stdout.write(b"$ ");
                stdout.flush().unwrap();

                match input::parse_input(buf.trim()) {
                    Ok(pipeline) => {
                        // Eval
                        let output = eval_pipeline(&pipeline);

                        // Print
                        match pipeline.stdout_writer {
                            Some(mut writer) => {
                                writer.write_all(&output.stdout).unwrap();
                                writer.flush().unwrap();
                            }
                            None => {
                                stdout.write_all(&output.stdout).unwrap();
                                stdout.flush().unwrap();
                            }
                        }

                        match pipeline.stderr_writer {
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

    fn get_repl_output(input: &[u8]) -> (String, String) {
        let mut reader = Cursor::new(input);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        start_repl(&mut reader, &mut stdout, &mut stderr);
        (
            // Remove the input prompt "$ " from stdout
            String::from_utf8(stdout).unwrap().replace("$ ", ""),
            String::from_utf8(stderr).unwrap(),
        )
    }

    fn assert_repl_output(input: &str, stdout: &str, stderr: &str) {
        let (result_stdout, result_stderr) = get_repl_output(input.as_bytes());
        assert!(
            stdout == result_stdout,
            "Input of '{}' expected stdout of '{}', instead got: '{}'",
            input.replace("\n", "\\n"),
            stdout.replace("\n", "\\n"),
            result_stdout.replace("\n", "\\n")
        );
        assert!(
            stderr == result_stderr,
            "Input of '{}' expected stderr of '{}', instead got: '{}'",
            input.replace("\n", "\\n"),
            stderr.replace("\n", "\\n"),
            result_stderr.replace("\n", "\\n")
        );
    }

    #[test]
    fn repl_prompt() {
        let mut reader = Cursor::new("");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        start_repl(&mut reader, &mut stdout, &mut stderr);
        let stdout_s = String::from_utf8(stdout).unwrap();
        let stderr_s = String::from_utf8(stderr).unwrap();
        assert!(stdout_s == "$ ", "Missing prompt '$' in stdout");
        assert!(stderr_s.is_empty(), "Should have no error with empty input");
    }

    #[test]
    fn repl_single_command() {
        assert_repl_output("echo 'hello world'\n", "hello world\n", "");
    }

    #[test]
    fn repl_multiple_commands() {
        assert_repl_output("echo first\necho second\n", "first\nsecond\n", "");
    }

    #[test]
    fn repl_empty_input() {
        assert_repl_output("\n", "", "");
    }

    #[test]
    fn repl_type_builtin() {
        assert_repl_output("type cd\n", "cd is a shell builtin\n", "");
    }

    #[test]
    fn repl_type_external() {
        assert_repl_output("type grep\n", "grep is /usr/bin/grep\n", "");
    }

    #[test]
    // TODO: make the program adhere to this (do error if nothing after pipe)
    fn repl_parse_error() {
        assert_repl_output("|\n", "", "");
    }

    #[test]
    fn repl_nonexisting_cmd() {
        assert_repl_output("non_existing\n", "", "non_existing: not found\n");
    }
}
