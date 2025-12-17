pub type Args = Vec<String>;

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

/// Handles quotes
fn parse_args(args: &str) -> Args {
    if args.is_empty() {
        return Default::default();
    }

    let mut output = Default::default();

    // Init character loop
    let mut chars = args.chars();
    let Some(mut prev_ch) = chars.next() else {
        return output;
    };
    let mut in_quotes = false;
    let mut word = String::with_capacity(args.len());
    if '\'' == prev_ch {
        in_quotes = true;
    } else {
        word.push(prev_ch);
    }

    // Loop through each character until we find connecting single quotes to collect into words
    // Don't include the quotes themselves
    for ch in chars {
        // println!();
        // dbg!(in_quotes);
        // dbg!(&word);
        // dbg!(&output);
        // dbg!(&prev_ch);
        // dbg!(&ch);
        match ch {
            ' ' => {
                // If not in a quoted word, then end current word on space
                if !in_quotes && !word.is_empty() {
                    output.push(word.clone());
                    word.clear();
                } 
                // Strip space between nonquoted words
                else if !word.is_empty() {
                    word.push(ch);
                } 
            }
            '\'' => {
                // End of quoted word
                if in_quotes {
                    output.push(word.clone());
                    word.clear();
                    in_quotes = false;
                }
                // Make sure to concat if previous was a quoted word, by removing from output and
                // starting to concat this new word
                else if '\'' == prev_ch && word.is_empty() {
                    word = output.pop().unwrap_or("".to_string());
                    in_quotes = true;
                }
                // Start of new quoted word
                else if ' ' == prev_ch {
                    // output.push(" ".to_string());
                    in_quotes = true;
                }
            }
            _ => word.push(ch),
        }

        prev_ch = ch;
    }

    // Add the final word
    if !word.is_empty() {
        output.push(word);
    }

    output
}
