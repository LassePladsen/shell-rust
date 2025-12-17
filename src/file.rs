use std::{fs, io, os::unix::fs::PermissionsExt};

const MODE_EXECUTABLE: u32 = 0o111;

pub fn is_executable_file(path: &str) -> io::Result<bool> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.is_file() && metadata.permissions().mode() & MODE_EXECUTABLE != 0)
}
