pub type Args<'a> = Vec<&'a str>;

/// Returns (cmd, vec of args)
pub fn parse_input<'a>(input: &'a str) -> (&'a str, Args<'a>) {
    let mut words = input.split_whitespace();
    let Some(cmd) = words.next() else {
        // If no cmd; don't do anything this iter. LP 2025-12-17
        return Default::default();
    };
    (cmd, parse_args(words))
}

fn parse_args<'a, I: Iterator<Item = &'a str>>(args: I) -> Args<'a> {
    args.collect()
}
