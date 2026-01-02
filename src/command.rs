use std::{
    error,
    fmt::{self, Debug, Display, Formatter},
    io::{self, Write},
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

#[allow(dead_code)]
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
}

pub struct Pipeline {
    pub commands: Vec<Command>,
    pub stdout: Option<Box<dyn Write>>, // Redirected stdout writer, None means use caller provided writer
    pub stderr: Option<Box<dyn Write>>, // Redirected stderr writer, None means use caller provided writer
}

impl Default for Pipeline {
    fn default() -> Self {
        Self {
            commands: Default::default(),
            stdout: None,
            stderr: None,
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
    PipeError(String),
    CommandNotFound(String),
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CommandError::Io(err) => write!(f, "{}", err),
            CommandError::PipeError(err) => write!(f, "{}", err),
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

pub fn run(cmd: &str, args: &[String], pipe_output: Option<Output>) -> Result<Output> {
    // Run my builtins
    if let Some(fn_) = builtin::get_builtin(cmd) {
        return Ok(fn_(args));
    }

    // Run external command
    if let Ok(paths) = env::get_paths()
        && let Ok(output) = spawn_ext_cmd(cmd, args, paths, pipe_output)
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
    pipe_output: Option<Output>,
) -> Result<Output> {
    if !cmd_in_paths(cmd, paths) {
        return Err(CommandError::CommandNotFound(format!(
            "Command {cmd} not found in path."
        )));
    }

    if pipe_output.is_none() {
        return Ok(std::process::Command::new(cmd).args(args).output()?.into());
    }

    // From here on out this is a piped command
    let pipe_output = pipe_output.unwrap();
    let mut child = std::process::Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()?;

    // Write output from previous command in pipeline to child stdin
    match child.stdin.take() {
        Some(mut stdin) => {
            _ = stdin.write(&pipe_output.stdout);
            Ok(child.wait_with_output()?.into())
        }
        None => Err(CommandError::PipeError(
            "Could not write piped output to new command child stdin".to_string(),
        )),
    }
}

fn notfound(cmd: &str) -> Output {
    Output {
        stderr: format!("{cmd}: not found\n").into(),
        ..Default::default()
    }
}
