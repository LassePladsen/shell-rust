use std::{
    error,
    fmt::{self, Debug, Display, Formatter},
    io::{self, Write, stderr, stdout},
    process::{self, Stdio},
};

use crate::file;
pub use crate::input::Args;
use crate::{env, input::ArgsSlice};

mod builtin;

#[derive(Default, Debug, Clone)]
pub struct Output {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl Output {
    pub fn is_empty(&self) -> bool {
        self.stdout.is_empty() && self.stderr.is_empty()
    }
}

impl From<process::Output> for Output {
    fn from(value: process::Output) -> Self {
        Self {
            stdout: value.stdout,
            stderr: value.stderr,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Command {
    pub args: Args,
    pub output: Output,
}

pub struct Pipeline {
    pub commands: Vec<Command>,
    pub stdout_writer: Box<dyn Write>,
    pub stderr_writer: Box<dyn Write>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self {
            commands: Default::default(),
            stdout_writer: Box::new(stdout()),
            stderr_writer: Box::new(stderr()),
        }
    }
}

impl Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("commands", &self.commands)
            .finish()
    }
}

impl Iterator for Pipeline {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        self.commands.clone().into_iter().next()
    }
}

type CommandFn = fn(ArgsSlice) -> Output;
type Result<T> = std::result::Result<T, CommandError>;

#[derive(Debug)]
pub enum CommandError {
    Io(io::Error),
    CommandNotFound(String),
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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

pub fn run(cmd: &str, args: &[String], stdin: Output) -> Result<Output> {
    // Run my builtins
    if let Some(fn_) = builtin::get_builtin(cmd) {
        return Ok(fn_(args));
    }

    // Run external command
    if let Ok(paths) = env::get_paths()
        && let Ok(output) = spawn_ext_cmd(cmd, args, paths, stdin)
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

pub fn spawn_ext_cmd(
    cmd: &str,
    args: ArgsSlice,
    paths: Vec<String>,
    stdin: Output,
) -> Result<Output> {
    if cmd_in_paths(cmd, paths) {
        let mut ext_cmd = process::Command::new(cmd);
        ext_cmd.args(args).stdin(Stdio::from(stdin.stdout));
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
