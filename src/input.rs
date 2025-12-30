use std::{
    default::Default,
    env,
    io::{Write, stderr, stdin, stdout},
    iter::Peekable,
    str::Chars,
};

pub type Input = Vec<String>;

pub struct CommandParams {
    pub input: Input,
    pub out_writer: Box<dyn Write>,
    pub err_writer: Box<dyn Write>,
}

impl Default for CommandParams {
    fn default() -> Self {
        Self {
            input: Default::default(),
            out_writer: Box::new(stdout()),
            err_writer: Box::new(stderr()),
        }
    }
}

/// Does a double pass: first it finds and collects the tokens, then it resolves the tokens to
/// strings doing e.g escaping, variable interpolation. (for separation of responsibility and testability)
pub fn parse_input(input: &str) -> CommandParams {
    let tokens = parse_to_tokens(input);
    resolve_tokens(tokens)
}

#[derive(Debug, Clone)]
enum Token {
    Literal(String),
    Variable(String),         // $VAR
    SingleQuoted(String),     // 'no expansion'
    DoubleQuoted(Vec<Token>), // "can have variables $VAR, escaped chars \", and single quotes ' inside"
    Whitespace,
}

fn parse_to_tokens(input: &str) -> Vec<Token> {
    if input.is_empty() {
        return Default::default();
    }

    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        tokens.push(match ch {
            '\'' => parse_single_quote(&mut chars),
            '"' => parse_double_quote(&mut chars),
            ch if ch.is_whitespace() => parse_whitespace(&mut chars),
            _ => parse_literal(&mut chars),
        })
    }

    tokens
}

fn parse_whitespace(chars: &mut Peekable<Chars>) -> Token {
    chars.next();
    Token::Whitespace
}

fn parse_single_quote(chars: &mut Peekable<Chars>) -> Token {
    chars.next(); // consume opening '
    let mut content = String::new();

    for ch in chars {
        // Stop the single quote token
        if ch == '\'' {
            break;
        }
        content.push(ch);
    }

    Token::SingleQuoted(content)
}

fn parse_double_quote(chars: &mut Peekable<Chars>) -> Token {
    chars.next(); // consume opening "
    let mut inner_tokens = Vec::new();
    let mut buf = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            // Stop the double quote token
            '"' => break,

            '\\' => {
                // Escape the next char if its escapable
                if let Some(next) = chars.next() {
                    match next {
                        // Escapable
                        '"' | '\\' | '$' | ' ' => buf.push(next),

                        _ => {
                            buf.push('\\');
                            buf.push(next);
                        }
                    }
                }
            }

            '$' => {
                // Save any literal content before the variable
                if !buf.is_empty() {
                    inner_tokens.push(Token::Literal(buf.clone()));
                    buf.clear();
                }

                // Parse variable name
                let var_name = parse_var_name(chars);
                inner_tokens.push(Token::Variable(var_name));
            }

            _ => buf.push(ch),
        }
    }

    if !buf.is_empty() {
        inner_tokens.push(Token::Literal(buf));
    }

    Token::DoubleQuoted(inner_tokens)
}

fn parse_literal(chars: &mut Peekable<Chars>) -> Token {
    let mut content = String::new();
    let mut escaped = false;
    while let Some(&ch) = chars.peek() {
        // Escape
        if !escaped && ch == '\\' {
            escaped = true;
            chars.next();
            continue;
        }

        // Replace ~ with home
        if !escaped && ch == '~' {
            match std::env::var("HOME") {
                Ok(s) => content.push_str(&s),
                Err(_) => content.push(ch),
            };
            chars.next();
            continue;
        }

        // Literal/normal string stops at unescaped whitespace or quote
        if !escaped && (ch.is_whitespace() || ch == '\'' || ch == '"') {
            break;
        }

        content.push(ch);
        chars.next();
        escaped = false;
    }

    Token::Literal(content)
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

fn resolve_tokens(tokens: Vec<Token>) -> CommandParams {
    let mut params = CommandParams::default();
    if tokens.is_empty() {
        return params;
    }

    let mut resolved_input = Input::default();
    let mut buf = String::new();

    for token in tokens {
        match token {
            Token::SingleQuoted(s) => buf.push_str(&s),
            Token::Variable(name) => resolve_variable(&mut buf, &name),
            Token::DoubleQuoted(inner_tokens) => resolve_double_quoted(&mut buf, &inner_tokens),
            Token::Whitespace => resolve_whitespace(&mut buf, &mut resolved_input),
            Token::Literal(s) => resolve_literal(&mut buf, &s),
        }
    }

    if !buf.is_empty() {
        resolved_input.push(buf);
    }

    params.input = resolved_input;
    params
}

fn resolve_literal(buf: &mut String, literal: &str) {
    // We need to check if the literal contains a redirection

    buf.push_str(literal)
}

fn resolve_whitespace(buf: &mut String, resolved_input: &mut Vec<String>) {
    // Separate tokens by a single space for all whitespace
    if !buf.is_empty() {
        resolved_input.push(buf.clone());
        buf.clear();
    }
}

fn resolve_variable(buf: &mut String, name: &str) {
    // If env variable not found it will resolve to nothing
    if let Ok(value) = env::var(name) {
        buf.push_str(&value);
    }
}

fn resolve_double_quoted(buf: &mut String, inner_tokens: &[Token]) {
    // resolve inner tokens
    for inner_token in inner_tokens {
        match inner_token {
            Token::Literal(s) => buf.push_str(s),
            Token::Variable(name) => {
                if let Ok(value) = env::var(name) {
                    buf.push_str(&value);
                }
            }
            _ => {} // shouldn't happen
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_double_quoted() {
        unsafe {
            std::env::set_var("myvar", "myvar_val");
        }
        let inner_tokens = [
            Token::Literal("Hello   after 3 spaces ".to_string()),
            Token::Variable("myvar".to_string()),
        ];
        let mut buf = String::new();
        super::resolve_double_quoted(&mut buf, &inner_tokens);
        assert_eq!(buf, "Hello   after 3 spaces myvar_val");
    }

    #[test]
    fn resolve_variable() {
        unsafe {
            std::env::set_var("myvar", "myvar_val");
        }
        let mut buf = String::new();
        super::resolve_variable(&mut buf, "myvar");
        assert_eq!(buf, "myvar_val");
    }

    #[test]
    fn resolve_whitespace() {
        let mut buf = "Hello   world".to_string();
        let mut buf2 = vec!["One".to_string(), "Two".to_string()];
        super::resolve_whitespace(&mut buf, &mut buf2);
        assert_eq!(buf2, ["One", "Two", "Hello   world"]);
    }

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
            super::parse_input("Hello   world").input,
            ["Hello", "world"]
        );
        assert_eq!(
            super::parse_input("myvar is: $myvar").input,
            ["myvar", "is:", "$myvar"]
        );
        assert_eq!(
            super::parse_input("cd ~/work").input,
            ["cd", "/home/myhome/work"]
        );

        // Single quotes
        assert_eq!(
            super::parse_input("'Hello   world'").input,
            ["Hello   world"]
        );
        assert_eq!(
            super::parse_input("'Hello   world'").input,
            ["Hello   world"]
        );
        assert_eq!(super::parse_input("'Hello''world'").input, ["Helloworld"]);
        assert_eq!(
            super::parse_input("'myvar is: $myvar'").input,
            ["myvar is: $myvar"]
        );
        assert_eq!(
            super::parse_input("myvar is: '$myvar'").input,
            ["myvar", "is:", "$myvar"]
        );
        assert_eq!(super::parse_input("'cd ~/work'").input, ["cd ~/work"]);
        assert_eq!(super::parse_input("cd '~/work'").input, ["cd", "~/work"]);

        // Double quotes
        assert_eq!(
            super::parse_input("\"Hello   world\"").input,
            ["Hello   world"]
        );
        assert_eq!(
            super::parse_input("\"Hello\"\"world\"").input,
            ["Helloworld"]
        );
        assert_eq!(
            super::parse_input("\"myvar is: $myvar\"").input,
            ["myvar is: myvar_val"]
        );
        assert_eq!(
            super::parse_input("myvar is: \"$myvar\"").input,
            ["myvar", "is:", "myvar_val"]
        );
        assert_eq!(super::parse_input("\"cd ~/work\"").input, ["cd ~/work"]);
        assert_eq!(super::parse_input("cd \"~/work\"").input, ["cd", "~/work"]);
    }
}
