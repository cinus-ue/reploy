#[derive(Debug)]
pub struct Status {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}