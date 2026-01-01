use std::io::{BufRead, Write};

use crate::command::{Output, Pipeline, run};
use crate::input::{self, ArgsSlice};

pub fn start_repl<R: BufRead, W1: Write, W2: Write>(
    reader: &mut R,
    stdout_writer: &mut W1,
    stderr_writer: &mut W2,
) {
    // Init
    _ = stdout_writer.write(b"$ ");
    stdout_writer.flush().unwrap();
    let mut buf = String::new();
    // Read
    loop {
        match reader.read_line(&mut buf) {
            Ok(0) => break, // EOF reached
            Ok(_) => {
                match input::parse_input(buf.trim()) {
                    Ok(mut pipeline) => {
                        // Eval
                        let output = eval_pipeline(&pipeline);
                        // Print
                        _ = pipeline.stdout_writer.write(&output.stdout);
                        _ = pipeline.stderr_writer.write(&output.stderr);
                        pipeline.stdout_writer.flush().unwrap();
                    }
                    Err(e) => {
                        let mut s = e.to_string();
                        s.push('\n');
                        // Print
                        _ = stderr_writer.write(s.as_bytes());
                    }
                };
                // Restart
                buf.clear();
                _ = stdout_writer.write(b"$ ");
                stdout_writer.flush().unwrap();
            }
            Err(_) => break, // Error reading
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

    fn run_repl(input: &[u8]) -> (String, String) {
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
        let (stdout_str, _) = run_repl(b"echo hello\n");
        assert!(
            stdout_str.starts_with("$ "),
            "stdout should start with prompt, got: {}",
            stdout_str
        );
        assert!(
            stdout_str.contains("hello"),
            "stdout should contain 'hello', got: {}",
            stdout_str
        );
    }

    #[test]
    fn repl_multiple_commands() {
        let (stdout_str, _) = run_repl(b"echo first\necho second\n");
        assert_eq!(
            stdout_str.matches("$ ").count(),
            2,
            "should have two prompts, got: {}",
            stdout_str
        );
        assert!(
            stdout_str.contains("first"),
            "stdout should contain 'first', got: {}",
            stdout_str
        );
        assert!(
            stdout_str.contains("second"),
            "stdout should contain 'second', got: {}",
            stdout_str
        );
    }

    #[test]
    fn repl_empty_input() {
        let (stdout_str, _) = run_repl(b"\n");
        assert_eq!(
            stdout_str.matches("$ ").count(),
            2,
            "should have initial prompt and one after empty line, got: {}",
            stdout_str
        );
    }

    #[test]
    fn repl_parse_error() {
        let (_, stderr_str) = run_repl(b"|\n");
        assert!(
            !stderr_str.is_empty(),
            "stderr should have error message, got: {}",
            stderr_str
        );
    }

    #[test]
    fn repl_initial_prompt() {
        let (stdout_str, _) = run_repl(b"");
        assert_eq!(
            stdout_str, "$ ",
            "should display initial prompt, got: {}",
            stdout_str
        );
    }

    #[test]
    fn repl_whitespace_handling() {
        let (stdout_str, _) = run_repl(b"  echo test  \n");
        assert!(
            stdout_str.contains("test"),
            "stdout should contain 'test' after trimming whitespace, got: {}",
            stdout_str
        );
    }

    #[test]
    fn repl_stderr_separation() {
        let (stdout_str, stderr_str) = run_repl(b"some_error_command\n");
        assert!(
            stdout_str.starts_with("$ "),
            "stdout should start with prompt, got stdout: {}, stderr: {}",
            stdout_str,
            stderr_str
        );
    }
}
