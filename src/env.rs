pub fn get_paths() -> Result<Vec<String>, std::env::VarError> {
    std::env::var("PATH").map(|s| s.split(":").map(|s2| s2.to_string()).collect())
}

