use crate::env;
use crate::file;

type Args<'a> = Vec<&'a str>;
type Cmd = fn(Args) -> String;

pub fn run(cmd: &str, args: Args) -> String {
    match get_cmd_builtin(cmd) {
        Some(fn_) => fn_(args),
        None => notfound(cmd),
    }
}

fn get_cmd_builtin(cmd: &str) -> Option<Cmd> {
    match cmd {
        "type" => Some(type_),
        "echo" => Some(echo),
        "exit" => Some(exit),
        _ => None,
    }
}

fn get_cmd_path(cmd: &str, paths: Vec<String>) -> Option<String> {
    for path in paths {
        let fullpath = format!("{path}/{cmd}");
        let Ok(executable) = file::is_executable_file(&fullpath) else {
            continue;
        };
        if executable {
            return Some(fullpath);
        }
    }
    None
}

fn type_(args: Args) -> String {
    let cmd = args.first().expect("Expected a command as argument");

    if get_cmd_builtin(cmd).is_some() {
        return format!("{cmd} is a shell builtin");
    }

    if let Ok(paths) = env::get_paths()
        && let Some(path) = get_cmd_path(cmd, paths)
    {
        return format!("{cmd} is {path}");
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
