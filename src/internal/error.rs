use std::fmt;
use std::io;
use ssh2;

#[derive(Debug)]
pub enum ReployError {
    Io(io::Error),
    Ssh(ssh2::Error),
    Runtime(String),
    AuthFailed,
    ConnectionFailed,
    CommandFailed(i32, String),
    InvalidRecipe(String),
    WithContext {
        source: Box<ReployError>,
        context: String,
    },
}

impl ReployError {
    pub fn with_context<S: Into<String>>(self, context: S) -> Self {
        ReployError::WithContext {
            source: Box::new(self),
            context: context.into(),
        }
    }
}

impl fmt::Display for ReployError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReployError::Io(e) => write!(f, "IO error: {}", e),
            ReployError::Ssh(e) => write!(f, "SSH error: {}", e),

            ReployError::Runtime(s) => write!(f, "Runtime error: {}", s),
            ReployError::AuthFailed => write!(f, "Authentication failed"),
            ReployError::ConnectionFailed => write!(f, "Connection failed"),
            ReployError::CommandFailed(code, msg) => 
                write!(f, "Command failed with exit code {}: {}", code, msg),
            ReployError::InvalidRecipe(s) => 
                write!(f, "Invalid recipe: {}", s),
            ReployError::WithContext { source, context } => 
                write!(f, "{}\nContext: {}", source, context),
        }
    }
}

impl std::error::Error for ReployError {}

impl From<io::Error> for ReployError {
    fn from(err: io::Error) -> Self {
        ReployError::Io(err)
    }
}

impl From<ssh2::Error> for ReployError {
    fn from(err: ssh2::Error) -> Self {
        ReployError::Ssh(err)
    }
}
