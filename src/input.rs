use std::env;

pub type Input = Vec<String>;

/// Does a double pass: first it finds and collects the tokens, then it resolves the tokens to
/// strings doing escaping, variable interpolation. (for abstraction and testability)
pub fn parse_input(input: &str) -> Input {
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
        match ch {
            '\'' => tokens.push(parse_single_quote(&mut chars)),
            '"' => tokens.push(parse_double_quote(&mut chars)),
            ch if ch.is_whitespace() => {
                chars.next();
                tokens.push(Token::Whitespace);
            }
            _ => tokens.push(parse_literal(&mut chars)),
        }
    }

    tokens
}

fn parse_single_quote(chars: &mut std::iter::Peekable<std::str::Chars>) -> Token {
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

fn parse_double_quote(chars: &mut std::iter::Peekable<std::str::Chars>) -> Token {
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

fn parse_literal(chars: &mut std::iter::Peekable<std::str::Chars>) -> Token {
    let mut content = String::new();

    while let Some(&ch) = chars.peek() {
        // Literal/normal string stops at whitespace or quote
        if ch.is_whitespace() || ch == '\'' || ch == '"' {
            break;
        }

        content.push(ch);
        chars.next();
    }

    Token::Literal(content)
}

fn parse_var_name(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
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

fn resolve_tokens(tokens: Vec<Token>) -> Input {
    if tokens.is_empty() {
        return Default::default();
    }

    let mut result = Input::default();
    let mut buf = String::new();

    for token in tokens {
        match token {
            Token::Literal(s) | Token::SingleQuoted(s) => buf.push_str(&s),

            Token::Variable(name) => {
                // If env variable not found it will resolve to nothing
                if let Ok(value) = env::var(&name) {
                    buf.push_str(&value);
                }
            }

            Token::DoubleQuoted(inner_tokens) => {
                // resolve inner tokens
                for inner_token in inner_tokens {
                    match inner_token {
                        Token::Literal(s) => buf.push_str(&s),
                        Token::Variable(name) => {
                            if let Ok(value) = env::var(&name) {
                                buf.push_str(&value);
                            }
                        }
                        _ => {} // shouldn't happen
                    }
                }
            }

            // Separate tokens by a single space for all whitespace
            Token::Whitespace => {
                if !buf.is_empty() {
                    result.push(buf.clone());
                    buf.clear();
                }
            }
        }
    }

    if !buf.is_empty() {
        result.push(buf);
    }

    result
}
