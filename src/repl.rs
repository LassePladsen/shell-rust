use std::io::Write;

use crate::command;
use crate::input::{self, Input};

use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

pub fn start_repl() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    const HISTORY_FILE: &str = "/tmp/.shell-rust-history.txt";
    _ = rl.load_history(HISTORY_FILE);
    rl.set_completion_type(rustyline::CompletionType::List);

    // The repl
    loop {
        // Read
        let readline = rl.readline("$ ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(&line)?;

                // Eval
                match input::parse_input(&line) {
                    Ok(mut params) => {
                        let (cmd, args) = (&params.input[0], &params.input[1..]);
                        let output = eval(cmd, args.to_vec());

                        // Print
                        _ = params.stdout.write_all(&output.stdout);
                        _ = params.stderr.write_all(&output.stderr);
                        params.stdout.flush().unwrap();
                    }
                    Err(err) => {
                        eprintln!("{err}");
                    }
                };
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("{err}");
                break;
            }
        }
    }

    if let Err(err) = rl.save_history(HISTORY_FILE) {
        eprintln!("{err}");
    }
    Ok(())
}

fn eval(cmd: &str, args: Input) -> command::Output {
    match command::run(cmd, args) {
        Ok(output) => output,
        Err(e) => command::Output {
            stderr: e.to_string().as_bytes().to_vec(),
            ..Default::default()
        },
    }
}
