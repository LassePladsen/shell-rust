use std::{default::Default, env, fmt::Debug, fs::File, io, iter::Peekable, mem, str::Chars};

use crate::command::{Command, Pipeline};

pub type Args = Vec<String>;
pub type ArgsSlice<'a> = &'a [String];

#[derive(Debug)]
enum Context {
    Normal,
    Escaped,
    SingleQuote,
    DoubleQuote,
}

/// Parses and resolves input in a single pass using a context system
pub fn parse_input(input: &str) -> io::Result<Pipeline> {
    let mut pipeline = Pipeline::default();
    let mut command = Command::default();
    if input.is_empty() {
        return Ok(pipeline);
    }

    let mut resolved_args = Vec::new();
    let mut buf = String::new();
    let mut context = Context::Normal;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match context {
            Context::Normal => handle_normal_context(
                ch,
                &mut chars,
                &mut buf,
                &mut resolved_args,
                &mut context,
                &mut command,
                &mut pipeline,
            )?,
            Context::Escaped => {
                // Any character is now literal
                buf.push(ch);
                context = Context::Normal;
            }
            Context::SingleQuote => handle_single_quote_context(ch, &mut buf, &mut context),
            Context::DoubleQuote => {
                handle_double_quote_context(ch, &mut chars, &mut buf, &mut context)
            }
        }
    }

    // Push any remaining content
    if !buf.is_empty() {
        resolved_args.push(buf);
    }

    // Add the last command
    command.args = resolved_args;
    pipeline.commands.push(command);

    Ok(pipeline)
}

fn handle_normal_context(
    ch: char,
    chars: &mut Peekable<Chars>,
    buf: &mut String,
    resolved_args: &mut Vec<String>,
    context: &mut Context,
    command: &mut Command,
    pipeline: &mut Pipeline,
) -> io::Result<()> {
    match ch {
        // Start new context
        '\'' => *context = Context::SingleQuote,
        '"' => *context = Context::DoubleQuote,
        '\\' => *context = Context::Escaped,

        // Expansion
        '~' => expand_tilde(buf),
        '$' => expand_variable(chars, buf),

        // Redirection & pipeline
        '>' => handle_redirection(chars, buf, pipeline)?,
        '|' => pipe(chars, buf, command, pipeline, context, resolved_args),

        _ if ch.is_whitespace() => separate_token(buf, resolved_args),
        _ => buf.push(ch),
    }
    Ok(())
}

fn pipe(
    chars: &mut Peekable<Chars>,
    buf: &mut String,
    params: &mut Command,
    pipeline: &mut Pipeline,
    context: &mut Context,
    resolved_args: &mut Vec<String>,
) {
    // This is the end of this command, add it to the pipeline and reset the states for the new piped command
    params.args = mem::take(resolved_args);
    pipeline.commands.push(mem::take(params));
    *buf = Default::default();
    *context = Context::Normal;

    chars.next();
}

fn handle_redirection(
    chars: &mut Peekable<Chars>,
    buf: &mut String,
    pipeline: &mut Pipeline,
) -> io::Result<()> {
    // Default file descriptor stdout
    let mut fd: u8 = 1;

    if let Some(ch) = buf.chars().next()
        && let Some(digit) = ch.to_digit(10)
    {
        fd = digit as u8;
    }

    let mut filename = String::new();
    let mut append = false;
    while let Some(ch) = chars.peek() {
        match ch {
            // Allow optional whitespace between '>' and filename, but stop at the whitespace after the filename
            ' ' => {
                if !filename.is_empty() {
                    break;
                } else {
                    chars.next();
                }
            }
            // '>>' means to append to file
            '>' => {
                append = true;
                chars.next();
            }
            _ => {
                filename.push(*ch);
                chars.next();
            }
        }
    }
    let writer = Box::new(
        File::options()
            .create(true)
            .write(true)
            .append(append)
            .open(filename)?,
    );

    match fd {
        1 => pipeline.stdout = writer,
        2 => pipeline.stderr = writer,
        _ => (),
    }
    buf.clear();

    Ok(())
}

fn handle_single_quote_context(ch: char, buf: &mut String, context: &mut Context) {
    if ch == '\'' {
        // End single quote
        *context = Context::Normal;
    } else {
        // Inside single quotes, everything is literal (no expansion)
        buf.push(ch);
    }
}

fn handle_double_quote_context(
    ch: char,
    chars: &mut Peekable<Chars>,
    buf: &mut String,
    context: &mut Context,
) {
    match ch {
        '"' => {
            // End double quote
            *context = Context::Normal;
        }
        '\\' => handle_escape_in_double_quote(chars, buf),
        '$' => expand_variable(chars, buf),
        _ => buf.push(ch),
    }
}

fn handle_escape_in_double_quote(chars: &mut Peekable<Chars>, buf: &mut String) {
    if let Some(&next) = chars.peek() {
        match next {
            '"' | '\\' | '$' | ' ' => {
                // Escapable characters
                chars.next();
                buf.push(next);
            }
            _ => {
                // Not escapable, keep the backslash
                buf.push('\\');
            }
        }
    } else {
        buf.push('\\');
    }
}

fn expand_tilde(buf: &mut String) {
    if let Ok(home) = env::var("HOME") {
        buf.push_str(&home);
    } else {
        buf.push('~');
    }
}

fn expand_variable(chars: &mut Peekable<Chars>, buf: &mut String) {
    let var_name = parse_var_name(chars);
    if let Ok(value) = env::var(&var_name) {
        buf.push_str(&value);
    }
}

fn separate_token(buf: &mut String, resolved_args: &mut Vec<String>) {
    if !buf.is_empty() {
        resolved_args.push(buf.clone());
        buf.clear();
    }
}

fn parse_var_name(chars: &mut Peekable<Chars>) -> String {
    let mut name = String::new();

    // ${name}
    if chars.peek() == Some(&'{') {
        chars.next(); // consume '{'
        for ch in chars.by_ref() {
            // Drop ending '}'
            if ch == '}' {
                break;
            }
            name.push(ch);
        }
        return name;
    }

    // $name
    while let Some(&ch) = chars.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            name.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    name
}

#[cfg(test)]
mod tests {
    use crate::input::Args;

    const TMP_FILE: &str = "/tmp/12138791273217897832798623798631.something";
    fn init_myvar() {
        unsafe {
            std::env::set_var("myvar", "myvar_val");
        }
        unsafe {
            std::env::set_var("HOME", "/home/myhome");
        }
    }

    fn get_args_from_parse_input(input: &str) -> Args {
        super::parse_input(input)
            .unwrap()
            .commands
            .first()
            .unwrap()
            .args
            .clone()
    }

    #[test]
    fn parse_input_normal() {
        init_myvar();
        assert_eq!(
            get_args_from_parse_input("Hello   world"),
            ["Hello", "world"]
        );
        assert_eq!(
            get_args_from_parse_input("myvar is: $myvar"),
            ["myvar", "is:", "myvar_val"]
        );
        assert_eq!(
            get_args_from_parse_input("cd ~/work"),
            ["cd", "/home/myhome/work"]
        );
    }

    #[test]
    fn parse_input_single_quote() {
        init_myvar();
        assert_eq!(
            get_args_from_parse_input("'Hello   world'"),
            ["Hello   world"]
        );
        assert_eq!(
            get_args_from_parse_input("'Hello   world'"),
            ["Hello   world"]
        );
        assert_eq!(get_args_from_parse_input("'Hello''world'"), ["Helloworld"]);
        assert_eq!(
            get_args_from_parse_input("'myvar is: $myvar'"),
            ["myvar is: $myvar"]
        );
        assert_eq!(
            get_args_from_parse_input("myvar is: '$myvar'"),
            ["myvar", "is:", "$myvar"]
        );
        assert_eq!(get_args_from_parse_input("'cd ~/work'"), ["cd ~/work"]);
        assert_eq!(get_args_from_parse_input("cd '~/work'"), ["cd", "~/work"]);
        assert_eq!(
            get_args_from_parse_input(&format!("echo hei '> {TMP_FILE}'")),
            ["echo", "hei", &format!("> {TMP_FILE}")]
        );
    }

    #[test]
    fn parse_input_double_quotes() {
        init_myvar();

        assert_eq!(
            get_args_from_parse_input("\"Hello   world\""),
            ["Hello   world"]
        );
        assert_eq!(
            get_args_from_parse_input("\"Hello\"\"world\""),
            ["Helloworld"]
        );
        assert_eq!(
            get_args_from_parse_input("\"myvar is: $myvar\""),
            ["myvar is: myvar_val"]
        );
        assert_eq!(
            get_args_from_parse_input("myvar is: \"$myvar\""),
            ["myvar", "is:", "myvar_val"]
        );
        assert_eq!(get_args_from_parse_input("\"cd ~/work\""), ["cd ~/work"]);
        assert_eq!(get_args_from_parse_input("cd \"~/work\""), ["cd", "~/work"]);
        assert_eq!(
            get_args_from_parse_input(&format!("echo hei \"> {TMP_FILE}\"")),
            ["echo", "hei", &format!("> {TMP_FILE}")]
        );
    }

    #[test]
    fn parse_input_redirection() {
        init_myvar();

        assert_eq!(
            get_args_from_parse_input(&format!("echo hei > {TMP_FILE}")),
            ["echo", "hei"]
        );
        assert_eq!(
            get_args_from_parse_input(&format!("echo hei >{TMP_FILE}")),
            ["echo", "hei"]
        );
        assert_eq!(
            get_args_from_parse_input(&format!("echo hei 2>{TMP_FILE}")),
            ["echo", "hei"]
        );
    }
}
