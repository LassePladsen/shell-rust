/// Returns (cmd, vec of args)
pub fn parse_input(input: &str) -> (&str, Vec<&str>) {
    let mut words = input.split_whitespace();
    let Some(cmd) = words.next() else {
        // If no cmd; don't do anything this iter. LP 2025-12-17
        return Default::default();
    };
    (cmd, words.collect())
}
