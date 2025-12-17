use std::{error, fmt, io, process};

use crate::env;
use crate::file;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CommandError {
    Io(io::Error),
    ExecutionFailed(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError::Io(err) => write!(f, "{}", err),
            CommandError::ExecutionFailed(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for CommandError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CommandError::Io(err) => Some(err),
            _ => None,
        }
    }
}
impl From<io::Error> for CommandError {
    fn from(err: io::Error) -> Self {
        CommandError::Io(err)
    }
}

type Args<'a> = Vec<&'a str>;
type Output = Vec<u8>;
type Cmd = fn(Args) -> Output;

pub fn run(cmd: &str, args: Args) -> Result<Output, CommandError> {
    if let Some(fn_) = get_cmd_builtin(cmd) {
        return Ok(fn_(args));
    }
    if let Ok(paths) = env::get_paths()
        && let Some(path) = get_cmd_path(cmd, paths)
    {
        let mut ext_cmd = process::Command::new(&path);
        for arg in args {
            ext_cmd.arg(arg);
        }
        let output = ext_cmd.output()?;
        if !output.status.success() {
            return Err(CommandError::ExecutionFailed(format!(
                "Execution returned nonsuccess exit code: {path}"
            )));
        }
        return Ok(output.stdout);
    }
    Ok(notfound(cmd))
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

fn type_(args: Args) -> Output {
    let cmd = args.first().expect("Expected a command as argument");

    if get_cmd_builtin(cmd).is_some() {
        return format!("{cmd} is a shell builtin").into();
    }

    if let Ok(paths) = env::get_paths()
        && let Some(path) = get_cmd_path(cmd, paths)
    {
        return format!("{cmd} is {path}").into();
    }

    notfound(cmd)
}

fn exit(args: Args) -> Output {
    process::exit(
        args.first()
            .map_or(0, |i| i.parse().expect("Expected integer exit code")),
    );
}

fn echo(args: Args) -> Output {
    args.join(" ").into()
}

fn notfound(cmd: &str) -> Output {
    format!("{cmd}: not found").into()
}
