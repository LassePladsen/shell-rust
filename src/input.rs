use std::{
    default::Default,
    env,
    fs::File,
    io::{self, Write, stderr, stdout},
    iter::Peekable,
    str::Chars,
};

pub type Args = Vec<String>;

pub struct CommandParams {
    pub args: Args,
    pub stdout: Box<dyn Write>,
    pub stderr: Box<dyn Write>,
}

impl Default for CommandParams {
    fn default() -> Self {
        Self {
            args: Default::default(),
            stdout: Box::new(stdout()),
            stderr: Box::new(stderr()),
        }
    }
}

enum Context {
    Normal,
    Escaped,
    SingleQuote,
    DoubleQuote,
}

/// Parses and resolves input in a single pass using a context system
pub fn parse_input(input: &str) -> io::Result<CommandParams> {
    let mut params = CommandParams::default();
    if input.is_empty() {
        return Ok(params);
    }

    let mut resolved_input = Vec::new();
    let mut buf = String::new();
    let mut context = Context::Normal;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match context {
            Context::Normal => handle_normal_context(
                ch,
                &mut chars,
                &mut buf,
                &mut resolved_input,
                &mut context,
                &mut params,
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
        resolved_input.push(buf);
    }

    params.args = resolved_input;
    Ok(params)
}

fn handle_normal_context(
    ch: char,
    chars: &mut Peekable<Chars>,
    buf: &mut String,
    resolved_input: &mut Vec<String>,
    context: &mut Context,
    params: &mut CommandParams,
) -> io::Result<()> {
    match ch {
        // Start new context
        '\'' => *context = Context::SingleQuote,
        '"' => *context = Context::DoubleQuote,
        '\\' => *context = Context::Escaped,

        // Expansion
        '~' => expand_tilde(buf),
        '$' => expand_variable(chars, buf),

        // Redirection
        '>' => handle_redirection(chars, buf, params)?,

        _ if ch.is_whitespace() => separate_token(buf, resolved_input),
        _ => buf.push(ch),
    }
    Ok(())
}

fn handle_redirection(
    chars: &mut Peekable<Chars>,
    buf: &mut String,
    params: &mut CommandParams,
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
        1 => params.stdout = writer,
        2 => params.stderr = writer,
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

fn separate_token(buf: &mut String, resolved_input: &mut Vec<String>) {
    if !buf.is_empty() {
        resolved_input.push(buf.clone());
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
    #[test]
    fn parse_input() {
        unsafe {
            std::env::set_var("myvar", "myvar_val");
        }
        unsafe {
            std::env::set_var("HOME", "/home/myhome");
        }

        const FILENAME: &str = "/tmp/12138791273217897832798623798631.something";

        // No quotes
        assert_eq!(
            super::parse_input("Hello   world").unwrap().args,
            ["Hello", "world"]
        );
        assert_eq!(
            super::parse_input("myvar is: $myvar").unwrap().args,
            ["myvar", "is:", "myvar_val"]
        );
        assert_eq!(
            super::parse_input("cd ~/work").unwrap().args,
            ["cd", "/home/myhome/work"]
        );

        // Single quotes
        assert_eq!(
            super::parse_input("'Hello   world'").unwrap().args,
            ["Hello   world"]
        );
        assert_eq!(
            super::parse_input("'Hello   world'").unwrap().args,
            ["Hello   world"]
        );
        assert_eq!(
            super::parse_input("'Hello''world'").unwrap().args,
            ["Helloworld"]
        );
        assert_eq!(
            super::parse_input("'myvar is: $myvar'").unwrap().args,
            ["myvar is: $myvar"]
        );
        assert_eq!(
            super::parse_input("myvar is: '$myvar'").unwrap().args,
            ["myvar", "is:", "$myvar"]
        );
        assert_eq!(
            super::parse_input("'cd ~/work'").unwrap().args,
            ["cd ~/work"]
        );
        assert_eq!(
            super::parse_input("cd '~/work'").unwrap().args,
            ["cd", "~/work"]
        );
        assert_eq!(
            super::parse_input(&format!("echo hei '> {FILENAME}'"))
                .unwrap()
                .args,
            ["echo", "hei", &format!("> {FILENAME}")]
        );

        // Double quotes
        assert_eq!(
            super::parse_input("\"Hello   world\"").unwrap().args,
            ["Hello   world"]
        );
        assert_eq!(
            super::parse_input("\"Hello\"\"world\"").unwrap().args,
            ["Helloworld"]
        );
        assert_eq!(
            super::parse_input("\"myvar is: $myvar\"").unwrap().args,
            ["myvar is: myvar_val"]
        );
        assert_eq!(
            super::parse_input("myvar is: \"$myvar\"").unwrap().args,
            ["myvar", "is:", "myvar_val"]
        );
        assert_eq!(
            super::parse_input("\"cd ~/work\"").unwrap().args,
            ["cd ~/work"]
        );
        assert_eq!(
            super::parse_input("cd \"~/work\"").unwrap().args,
            ["cd", "~/work"]
        );
        assert_eq!(
            super::parse_input(&format!("echo hei \"> {FILENAME}\""))
                .unwrap()
                .args,
            ["echo", "hei", &format!("> {FILENAME}")]
        );

        // Redirection
        assert_eq!(
            super::parse_input(&format!("echo hei > {FILENAME}"))
                .unwrap()
                .args,
            ["echo", "hei"]
        );
        assert_eq!(
            super::parse_input(&format!("echo hei >{FILENAME}"))
                .unwrap()
                .args,
            ["echo", "hei"]
        );
        assert_eq!(
            super::parse_input(&format!("echo hei 2>{FILENAME}"))
                .unwrap()
                .args,
            ["echo", "hei"]
        );

        _ = std::fs::remove_file(FILENAME);
    }
}
