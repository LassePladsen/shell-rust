use std::env::{self, VarError};

pub fn get_paths() -> Result<Vec<String>, VarError> {
    env::var("PATH").map(|s| s.split(":").map(|s2| s2.to_string()).collect())
}
