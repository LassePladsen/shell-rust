use std::{env, error, fmt, fs, io, os::unix::fs::PermissionsExt};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Env(env::VarError),
    IsNotADirectory(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "{}", err),
            Error::Env(err) => write!(f, "{}", err),
            Error::IsNotADirectory(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<env::VarError> for Error {
    fn from(err: env::VarError) -> Self {
        Error::Env(err)
    }
}

type Result<T> = std::result::Result<T, Error>;

const MODE_EXECUTABLE: u32 = 0o111;

pub fn is_executable_file(path: &str) -> Result<bool> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.is_file() && metadata.permissions().mode() & MODE_EXECUTABLE != 0)
}

pub fn resolve_path(path: &str) -> Result<String> {
    // Resolve tilde '~' to $HOME
    let home = std::env::var("HOME")?;
    let resolved_path = path.replace("~", &home);

    // Resolve to abs path and check if is dir
    let abs_path = std::path::absolute(&resolved_path)?;
    let metadata = std::fs::metadata(&abs_path)?;

    if metadata.is_dir() {
        Ok(abs_path.to_string_lossy().into())
    } else {
        Err(Error::IsNotADirectory(format!(
            "{path}: No such file or directory"
        )))
    }
}
