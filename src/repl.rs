use std::io::{self, Write};

use crate::command;
use crate::input::{self, Input};

pub fn start_repl<R: io::BufRead>(reader: &mut R) {
    // Init
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();

    // Read
    while let Ok(raw_input) = read_line(reader, &mut buf) {
        let input = input::parse_input(raw_input);
        let (cmd, args) = (&input[0], input[1..].to_vec());

        // Eval
        let output = eval(cmd, args.to_vec());

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

fn eval(cmd: &str, args: Input) -> String {
    match command::run(cmd, args) {
        Ok(output) => str::from_utf8(&output)
            .expect("Could not write bytes to string")
            .to_string(),
        Err(e) => e.to_string(),
    }
}
