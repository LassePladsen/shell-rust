use super::{Args, Cmd, Output};
use crate::env;

pub fn get_cmd(cmd: &str) -> Option<Cmd> {
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
    let Some(path) = args.first() else {
        return Default::default();
    };

    let err_msg = format!("cd: {path}: No such file or directory\n").into();

    // Abs path
    let Ok(metadata) = std::fs::metadata(path) else {
        return err_msg;
    };

    // Not directory
    if !metadata.is_dir() {
        return err_msg;
    }

    // Change to dir
    match std::env::set_current_dir(path) {
        Ok(_) => Default::default(),
        Err(err) => format!("cd: could not change directory to {path}: {err}\n").into(),
    }
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

fn exit(args: Args) -> Output {
    std::process::exit(
        args.first()
            .map_or(0, |i| i.parse().expect("Expected integer exit code")),
    );
}

fn echo(args: Args) -> Output {
    format!("{}\n", args.join(" ")).into()
}
