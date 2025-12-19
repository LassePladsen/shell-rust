use crate::env;
use crate::file;
use crate::args::Args;
use crate::command::{CommandFn, Output};

pub fn get_cmd(cmd: &str) -> Option<CommandFn> {
    match cmd {
        "type" => Some(type_),
        "echo" => Some(echo),
        "exit" => Some(exit),
        "pwd" => Some(pwd),
        "cd" => Some(cd),
        _ => None,
    }
}

fn cd(args: Args) -> Output {
    let path = match args.first() {
        Some(path) => path,
        None => "~", // Defaults to cd'ing home if no args
    };

    if let Ok(abs_path) = file::resolve_path(path)
        && let Ok(_) = std::env::set_current_dir(abs_path)
    {
        return Default::default();
    }

    format!("cd: {path}: No such file or directory\n").into()
}

fn pwd(_args: Args) -> Output {
    match std::env::current_dir() {
        Ok(pathbuf) => format!("{}\n", pathbuf.to_string_lossy()).into(),
        Err(_) => "Unable to get cwd from std::env::current_dir\n".into(),
    }
}

fn type_(args: Args) -> Output {
    let Some(cmd) = args.first() else {
        return Default::default();
    };

    if get_cmd(cmd).is_some() {
        return format!("{cmd} is a shell builtin\n").into();
    }

    if let Ok(paths) = env::get_paths()
        && let Some(path) = super::get_cmd_path(cmd, paths)
    {
        return format!("{cmd} is {path}\n").into();
    }

    super::notfound(cmd)
}

fn echo(args: Args) -> Output {
    format!("{}\n", args.join(" ")).into()
}

fn exit(args: Args) -> Output {
    std::process::exit(
        args.first()
            .map_or(0, |i| i.parse().expect("Expected integer exit code")),
    );
}

