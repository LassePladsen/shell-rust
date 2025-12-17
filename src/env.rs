use std::env;

pub fn get_paths() -> Result<Vec<String>, std::env::VarError> {
    env::var("PATH").map(|s| s.split(":").map(|s2| s2.to_string()).collect())
}
