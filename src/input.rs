use std::{
    default::Default,
    env,
    fs::File,
    io::{self, Write, stderr, stdout},
    iter::Peekable,
    str::Chars,
};

pub type Input = Vec<String>;

pub struct CommandParams {
    pub input: Input,
    pub stdout: Box<dyn Write>,
    pub stderr: Box<dyn Write>,
}

impl Default for CommandParams {
    fn default() -> Self {
        Self {
            input: Default::default(),
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

    params.input = resolved_input;
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
        // Allow optional whitespace between '>' and filename, but stop at the whitespace after the filename
        if *ch == ' ' {
            if !filename.is_empty() {
                break;
            } else {
                chars.next();
                continue;
            }
        } 
        // '>>' means to append to file
        else if *ch == '>' {
            append = true;
            chars.next();
            continue;
        }

        filename.push(*ch);
        chars.next();
    }
    let writer = Box::new(File::options().append(append).create(true).open(filename)?);

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

        // No quotes
        assert_eq!(
            super::parse_input("Hello   world").unwrap().input,
            ["Hello", "world"]
        );
        assert_eq!(
            super::parse_input("myvar is: $myvar").unwrap().input,
            ["myvar", "is:", "myvar_val"]
        );
        assert_eq!(
            super::parse_input("cd ~/work").unwrap().input,
            ["cd", "/home/myhome/work"]
        );

        // Single quotes
        assert_eq!(
            super::parse_input("'Hello   world'").unwrap().input,
            ["Hello   world"]
        );
        assert_eq!(
            super::parse_input("'Hello   world'").unwrap().input,
            ["Hello   world"]
        );
        assert_eq!(
            super::parse_input("'Hello''world'").unwrap().input,
            ["Helloworld"]
        );
        assert_eq!(
            super::parse_input("'myvar is: $myvar'").unwrap().input,
            ["myvar is: $myvar"]
        );
        assert_eq!(
            super::parse_input("myvar is: '$myvar'").unwrap().input,
            ["myvar", "is:", "$myvar"]
        );
        assert_eq!(
            super::parse_input("'cd ~/work'").unwrap().input,
            ["cd ~/work"]
        );
        assert_eq!(
            super::parse_input("cd '~/work'").unwrap().input,
            ["cd", "~/work"]
        );

        // Double quotes
        assert_eq!(
            super::parse_input("\"Hello   world\"").unwrap().input,
            ["Hello   world"]
        );
        assert_eq!(
            super::parse_input("\"Hello\"\"world\"").unwrap().input,
            ["Helloworld"]
        );
        assert_eq!(
            super::parse_input("\"myvar is: $myvar\"").unwrap().input,
            ["myvar is: myvar_val"]
        );
        assert_eq!(
            super::parse_input("myvar is: \"$myvar\"").unwrap().input,
            ["myvar", "is:", "myvar_val"]
        );
        assert_eq!(
            super::parse_input("\"cd ~/work\"").unwrap().input,
            ["cd ~/work"]
        );
        assert_eq!(
            super::parse_input("cd \"~/work\"").unwrap().input,
            ["cd", "~/work"]
        );
    }
}
