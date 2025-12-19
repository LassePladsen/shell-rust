mod args;
pub use args::Args;

/// Returns (cmd, vec of args)
pub fn parse_input(input: &str) -> (&str, args::Args) {
    if input.is_empty() {
        return (input, Default::default());
    }
    match input.split_once(" ") {
        Some((cmd, arg_str)) => (cmd, args::parse_args(arg_str)),
        None => (input, Default::default()), // no args
    }
}
