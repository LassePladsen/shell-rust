use std::{
    default::Default,
    env,
    fmt::Debug,
    fs::File,
    io::{self, Write, stderr, stdout},
    iter::Peekable,
    mem,
    str::Chars,
};

#[derive(Debug, Default, Clone)]
pub struct Command {
    pub args: Args,
}

pub struct CommandPipeline {
    pub commands: Vec<Command>,
    pub stdout: Box<dyn Write>,
    pub stderr: Box<dyn Write>,
}

impl Debug for CommandPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandPipeline")
            .field("commands", &self.commands)
            .finish()
    }
}

impl Iterator for CommandPipeline {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        self.commands.clone().into_iter().next()
    }
}

pub type Args = Vec<String>;

#[derive(Debug)]
enum Context {
    Normal,
    Escaped,
    SingleQuote,
    DoubleQuote,
}

/// Parses and resolves input in a single pass using a context system
pub fn parse_input(input: &str) -> io::Result<CommandPipeline> {
    let mut pipeline = CommandPipeline::default();
    let mut params = Command::default();
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
                &mut params,
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
    params.args = resolved_args;
    pipeline.push(params);

    Ok(pipeline)
}

fn handle_normal_context(
    ch: char,
    chars: &mut Peekable<Chars>,
    buf: &mut String,
    resolved_args: &mut Vec<String>,
    context: &mut Context,
    params: &mut Command,
    pipeline: &mut CommandPipeline,
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
        '>' => handle_redirection(chars, buf, params)?,
        '|' => pipe(chars, buf, params, pipeline, context, resolved_args),

        _ if ch.is_whitespace() => separate_token(buf, resolved_args),
        _ => buf.push(ch),
    }
    Ok(())
}

fn pipe(
    chars: &mut Peekable<Chars>,
    buf: &mut String,
    params: &mut Command,
    pipeline: &mut CommandPipeline,
    context: &mut Context,
    resolved_args: &mut Vec<String>,
) {
    // This is the end of this command, add it to the pipeline and reset the states for the new piped command
    params.args = mem::take(resolved_args);
    pipeline.push(mem::take(params));
    *buf = Default::default();
    *context = Context::Normal;

    chars.next();
}

fn handle_redirection(
    chars: &mut Peekable<Chars>,
    buf: &mut String,
    params: &mut Command,
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
            super::parse_input("Hello   world")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["Hello", "world"]
        );
        assert_eq!(
            super::parse_input("myvar is: $myvar")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["myvar", "is:", "myvar_val"]
        );
        assert_eq!(
            super::parse_input("cd ~/work")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["cd", "/home/myhome/work"]
        );

        // Single quotes
        assert_eq!(
            super::parse_input("'Hello   world'")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["Hello   world"]
        );
        assert_eq!(
            super::parse_input("'Hello   world'")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["Hello   world"]
        );
        assert_eq!(
            super::parse_input("'Hello''world'")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["Helloworld"]
        );
        assert_eq!(
            super::parse_input("'myvar is: $myvar'")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["myvar is: $myvar"]
        );
        assert_eq!(
            super::parse_input("myvar is: '$myvar'")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["myvar", "is:", "$myvar"]
        );
        assert_eq!(
            super::parse_input("'cd ~/work'")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["cd ~/work"]
        );
        assert_eq!(
            super::parse_input("cd '~/work'")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["cd", "~/work"]
        );
        assert_eq!(
            super::parse_input(&format!("echo hei '> {FILENAME}'"))
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["echo", "hei", &format!("> {FILENAME}")]
        );

        // Double quotes
        assert_eq!(
            super::parse_input("\"Hello   world\"")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["Hello   world"]
        );
        assert_eq!(
            super::parse_input("\"Hello\"\"world\"")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["Helloworld"]
        );
        assert_eq!(
            super::parse_input("\"myvar is: $myvar\"")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["myvar is: myvar_val"]
        );
        assert_eq!(
            super::parse_input("myvar is: \"$myvar\"")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["myvar", "is:", "myvar_val"]
        );
        assert_eq!(
            super::parse_input("\"cd ~/work\"")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["cd ~/work"]
        );
        assert_eq!(
            super::parse_input("cd \"~/work\"")
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["cd", "~/work"]
        );
        assert_eq!(
            super::parse_input(&format!("echo hei \"> {FILENAME}\""))
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["echo", "hei", &format!("> {FILENAME}")]
        );

        // Redirection
        assert_eq!(
            super::parse_input(&format!("echo hei > {FILENAME}"))
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["echo", "hei"]
        );
        assert_eq!(
            super::parse_input(&format!("echo hei >{FILENAME}"))
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["echo", "hei"]
        );
        assert_eq!(
            super::parse_input(&format!("echo hei 2>{FILENAME}"))
                .unwrap()
                .first()
                .unwrap()
                .args,
            ["echo", "hei"]
        );

        _ = std::fs::remove_file(FILENAME);
    }
}
