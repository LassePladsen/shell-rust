type Args<'a> = Vec<&'a str>;
type Cmd = fn(Args);

pub fn get_cmd(cmd: &str) -> Option<Cmd> {
    match cmd {
        "type" => Some(type_),
        "echo" => Some(echo),
        "exit" => Some(exit),
        _ => None,
    }
}

pub fn run(cmd: &str, args: Args) {
    match get_cmd(cmd) {
        Some(fn_) => fn_(args),
        None => notfound(cmd),
    }
}

fn type_(args: Args) {
    let cmd = args.first().expect("Expected a command as argument");
    match get_cmd(cmd) {
        Some(_) => println!("{cmd} is a shell builtin"),
        None => notfound(cmd),

    }
}

fn exit(args: Args) {
    std::process::exit(
        args.first()
            .map_or(0, |i| i.parse().expect("Expected integer exit code")),
    )
}

fn echo(args: Args) {
    println!("{}", args.join(" "));
}

fn notfound(cmd: &str) {
    println!("{cmd}: not found");
}
