use std::path::PathBuf;

pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

pub fn ssh_key() -> PathBuf {
    home_dir()
        .map(|d| d.join(".ssh").join("id_rsa"))
        .unwrap_or(PathBuf::new())
}
