use std::io::{self, Write};

use crate::cmd;

pub fn start_repl<R: io::BufRead>(reader: &mut R) {
    // Init
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    let mut cmd: &str;

    // REPL
    while read_line(reader, &mut buf).is_ok() {
        // Read
        cmd = buf.trim();

        eval(cmd);

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

fn eval(input: &str) {
    let mut words = input.split_whitespace();
    let cmd = words.next().expect("Could not find command in the input: '{input}'");
    let args: Vec<&str> = words.collect();
    cmd::run(cmd, args);
}
