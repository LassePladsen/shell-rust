pub type Args = Vec<String>;

#[derive(Debug)]
enum RawArgKind {
    SingleQuote,
    DoubleQuote,
    Space,
    Literal, // everything else, aka. normal string
}
use RawArgKind::*;

#[derive(Debug)]
struct RawArg {
    kind: RawArgKind,
    content: String,
}

#[derive(Debug)]
struct Single

impl RawArg {
    fn push_char(&self, ch: char) {
        match self.kind {
            SingleQuote: 
        }
    }
    
}

type RawArgs = Vec<RawArg>;

/// Returns (cmd, vec of args)
pub fn parse_input(input: &str) -> (&str, Args) {
    if input.is_empty() {
        return (input, Default::default());
    }
    match input.split_once(" ") {
        Some((cmd, arg_str)) => (cmd, parse_args(arg_str)),
        None => (input, Default::default()), // no args
    }
}

fn parse_args(args: &str) -> Args {
    if args.is_empty() {
        return Default::default();
    }
    resolve_raw_args(collect_raw_args(args))
}

fn collect_raw_args(args: &str) -> RawArgs {
    if args.is_empty() {
        return Default::default();
    }

    let mut chars = args.chars();
    let mut output: RawArgs = Default::default();

    if args.len() == 1 {
        // Treat as Literal. Doesn't matter if its technically whitespace.
        output.push(RawArg {
            kind: RawArgKind::Literal,
            content: chars
                .next()
                .expect("Should be one char in this for sure")
                .into(),
        });
        return output;
    }

    // Loop through each character, start a single/double quoted section until the next unescaped
    // corresponding quote. Don't include the quotes themselves except if inside a quoted section
    // or escaped.
    let mut kind: Option<RawArgKind> = None;
    let mut raw_arg: RawArg;
    for ch in chars {
        dbg!(&ch);

        // Between args
        if kind.is_none() {
            raw_arg = RawArg {
                kind: match ch {
                    '\'' => SingleQuote,
                    '"' => DoubleQuote,
                    ' ' => Space,
                    _ => Literal,
                },
                content: Default::default(),
            };
            continue;
        }

        raw_arg.push_char(ch);







        {
            ' ' => {
                // If not in a quoted word, then end current word on space
                if !in_quotes && !arg_buf.is_empty() {
                    output.push(arg_buf.clone());
                    arg_buf.clear();
                }
                // Strip space between nonquoted words
                else if !arg_buf.is_empty() {
                    arg_buf.push(ch);
                }
            }
            '\'' => {
                // End of quoted word
                if in_quotes {
                    output.push(arg_buf.clone());
                    arg_buf.clear();
                    in_quotes = false;
                }
                // Make sure to concat if previous was a quoted word, by removing from output and
                // starting to concat this new word
                else if '\'' == prev_ch && arg_buf.is_empty() {
                    arg_buf = output.pop().unwrap_or("".to_string());
                    in_quotes = true;
                }
                // Start of new quoted word
                else if ' ' == prev_ch {
                    // output.push(" ".to_string());
                    in_quotes = true;
                }
            }
            _ => arg_buf.push(ch),
        }

        prev_ch = ch;
    }

    // Add the final word
    if !arg_buf.is_empty() {
        output.push(arg_buf);
    }

    output
}

fn resolve_raw_args(raw_args: RawArgs) -> Args {
    todo!()
}
