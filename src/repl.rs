use std::io::{self, Write};

use crate::args::{self, Args};
use crate::command;

pub fn start_repl<R: io::BufRead>(reader: &mut R) {
    // Init
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    let mut output;

    // Read
    while let Ok(input) = read_line(reader, &mut buf) {
        let (cmd, args) = parse_input(input);

        // Eval
        output = eval(cmd, args);

        // Print
        print!("{output}");

        // Restart
        buf.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
    }
}

fn read_line<'a, R: io::BufRead>(reader: &mut R, buf: &'a mut String) -> io::Result<&'a str> {
    reader.read_line(buf)?;
    Ok(buf.trim())
}

/// Returns (cmd, vec of args)
pub fn parse_input(input: &str) -> (&str, Args) {
    if input.is_empty() {
        return (input, Default::default());
    }
    match input.split_once(" ") {
        Some((cmd, arg_str)) => (cmd, args::parse_args(arg_str)),
        None => (input, Default::default()), // no args
    }
}

fn eval(cmd: &str, args: Args) -> String {
    match command::run(cmd, args) {
        Ok(output) => str::from_utf8(&output)
            .expect("Could not write bytes to string")
            .to_string(),
        Err(e) => e.to_string(),
    }
}
