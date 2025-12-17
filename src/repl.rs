use std::io::{self, Write};

use crate::cmd;
use crate::input;

pub fn start_repl<R: io::BufRead>(reader: &mut R) {
    // Init
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    let mut output;

    // REPL

    // read
    while let Ok(input) = read_line(reader, &mut buf) {
        // Print
        output = eval(input);
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

fn eval(input: &str) -> String {
    let (cmd, args) = input::parse_input(input);
    match cmd::run(cmd, args) {
        Ok(output) => str::from_utf8(&output)
            .expect("Could not write bytes to string")
            .to_string(),
        Err(e) => e.to_string(),
    }
}
