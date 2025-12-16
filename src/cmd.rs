type Args<'a>= Vec<&'a str>;

pub fn exit() {
    std::process::exit(0)
}
pub fn echo(args: Args) {
    println!("{}", args.join(" "));
}
pub fn notfound(cmd: &str) {
    println!("{cmd}: command not found");
}
