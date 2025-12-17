type Args<'a> = Vec<&'a str>;
type Cmd = fn(Args) -> String;

pub fn run(cmd: &str, args: Args) -> String {
    match get_builtin(cmd) {
        Some(fn_) => fn_(args),
        None => notfound(cmd),
    }
}

fn get_builtin(cmd: &str) -> Option<Cmd> {
    match cmd {
        "type" => Some(type_),
        "echo" => Some(echo),
        "exit" => Some(exit),
        _ => None,
    }
}

fn type_(args: Args) -> String {
    let cmd = args.first().expect("Expected a command as argument");
    if get_builtin(cmd).is_some() {
        return format!("{cmd} is a shell builtin");
    }

    notfound(cmd)
}

fn exit(args: Args) -> String {
    std::process::exit(
        args.first()
            .map_or(0, |i| i.parse().expect("Expected integer exit code")),
    );
}

fn echo(args: Args) -> String {
    args.join(" ").to_string()
}

fn notfound(cmd: &str) -> String {
    format!("{cmd}: not found")
}
