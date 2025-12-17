use std::{error, fmt, io};

use crate::env;
use crate::file;

mod builtin;

#[derive(Debug)]
pub enum CmdError {
    Io(io::Error),
    CmdNotFound(String),
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CmdError::Io(err) => write!(f, "{}", err),
            CmdError::CmdNotFound(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for CmdError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            CmdError::Io(err) => Some(err),
            _ => None,
        }
    }
}
impl From<io::Error> for CmdError {
    fn from(err: io::Error) -> Self {
        CmdError::Io(err)
    }
}

type Args<'a> = Vec<&'a str>;
type Output = Vec<u8>;
type Cmd = fn(Args) -> Output;
type Result<T> = std::result::Result<T, CmdError>;

pub fn run(cmd: &str, args: Args) -> Result<Output> {
    // Run my builtins
    if let Some(fn_) = builtin::get_cmd(cmd) {
        return Ok(fn_(args));
    }

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

pub fn spawn_ext_cmd(cmd: &str, args: Args, paths: Vec<String>) -> Result<Output> {
    if get_cmd_path(cmd, paths).is_some() {
        let mut ext_cmd = std::process::Command::new(cmd);
        for arg in args {
            ext_cmd.arg(arg);
        }
        let output = ext_cmd.output()?;
        return Ok(if !output.stdout.is_empty() {
            output.stdout
        } else {
            output.stderr
        });
    }
    Err(CmdError::CmdNotFound(format!(
        "Command {cmd} not found in path."
    )))
}

fn notfound(cmd: &str) -> Output {
    format!("{cmd}: not found\n").into()
}
