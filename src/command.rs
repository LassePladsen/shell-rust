use std::{error, fmt, io, process};

use crate::env;
use crate::file;
pub use crate::input::Args;

mod builtin;

#[derive(Default)]
pub struct Output {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl From<process::Output> for Output {
    fn from(value: process::Output) -> Self {
        Self {
            stdout: value.stdout,
            stderr: value.stderr,
        }
    }
}

type CommandFn = fn(Args) -> Output;
type Result<T> = std::result::Result<T, CommandError>;

#[derive(Debug)]
pub enum CommandError {
    Io(io::Error),
    CommandNotFound(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError::Io(err) => write!(f, "{}", err),
            CommandError::CommandNotFound(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for CommandError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
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

pub fn run(cmd: &str, args: Args) -> Result<Output> {
    // Run my builtins
    if let Some(fn_) = builtin::get_cmd(cmd) {
        return Ok(fn_(args));
    }

    // Run external command
    if let Ok(paths) = env::get_paths()
        && let Ok(output) = spawn_ext_cmd(cmd, args, paths)
    {
        return Ok(output);
    }
    Ok(notfound(cmd))
}

pub fn get_cmd_path(cmd: &str, paths: Vec<String>) -> Option<String> {
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

pub fn cmd_in_paths(cmd: &str, paths: Vec<String>) -> bool {
    get_cmd_path(cmd, paths).is_some()
}

pub fn spawn_ext_cmd(cmd: &str, args: Args, paths: Vec<String>) -> Result<Output> {
    if cmd_in_paths(cmd, paths) {
        let mut ext_cmd = std::process::Command::new(cmd);
        ext_cmd.args(args);
        return Ok(ext_cmd.output()?.into());
    }

    Err(CommandError::CommandNotFound(format!(
        "Command {cmd} not found in path."
    )))
}

fn notfound(cmd: &str) -> Output {
    Output {
        stdout: format!("{cmd}: not found\n").into(),
        ..Default::default()
    }
}
