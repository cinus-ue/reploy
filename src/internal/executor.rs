use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Read};
use std::net::TcpStream;
use std::path::Path;

use internal::error::ReployError;
use internal::Stdio;

use super::util;

const BUF_SIZE: usize = 1024 * 1024;

pub trait Executor {
    fn connect(&mut self, target: &str) -> Result<(), ReployError>;
    fn disconnect(&mut self) -> Result<(), ReployError>;
    fn execute(&mut self, command: &str) -> Result<(), ReployError>;
    fn send(&self, source: &str, dest: &str) -> Result<(), ReployError>;
    fn recv(&self, source: &str, dest: &str) -> Result<(), ReployError>;
    fn stdio(&self) -> &Stdio;
}

#[derive(Debug)]
pub struct LocalExecutor {
    stdio: Stdio,
}

impl LocalExecutor {
    pub fn new() -> Self {
        LocalExecutor {
            stdio: Stdio::default(),
        }
    }
}

impl Executor for LocalExecutor {
    fn connect(&mut self, _target: &str) -> Result<(), ReployError> {
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), ReployError> {
        Ok(())
    }

    fn execute(&mut self, command: &str) -> Result<(), ReployError> {
        use std::process::Command;

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", command])
                .output()
                .map_err(|e| {
                    ReployError::CommandFailed(
                        -1,
                        format!(
                            "Failed to execute command on Windows: {}, Error: {}",
                            command, e
                        ),
                    )
                })?
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .map_err(|e| {
                    ReployError::CommandFailed(
                        -1,
                        format!(
                            "Failed to execute command on Unix: {}, Error: {}",
                            command, e
                        ),
                    )
                })?
        };

        self.stdio = Stdio {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        };

        Ok(())
    }

    fn send(&self, source: &str, dest: &str) -> Result<(), ReployError> {
        let source_path = Path::new(source);
        let dest_path = Path::new(dest);
        std::fs::copy(source_path, dest_path)?;
        Ok(())
    }

    fn recv(&self, source: &str, dest: &str) -> Result<(), ReployError> {
        let source_path = Path::new(source);
        let dest_path = Path::new(dest);
        std::fs::copy(source_path, dest_path)?;
        Ok(())
    }

    fn stdio(&self) -> &Stdio {
        &self.stdio
    }
}

pub struct SshExecutor {
    session: ssh2::Session,
    stdio: Stdio,
    identity: std::path::PathBuf,
}

impl SshExecutor {
    pub fn new() -> Self {
        SshExecutor {
            session: ssh2::Session::new().unwrap(),
            stdio: Stdio::default(),
            identity: util::ssh_key(),
        }
    }

    pub fn set_identity(&mut self, identity: &str) {
        self.identity = std::path::PathBuf::from(identity);
    }
}

impl Executor for SshExecutor {
    fn connect(&mut self, target: &str) -> Result<(), ReployError> {
        let mut user = "root";
        let mut port = "22";
        let mut host;

        if target.contains("@") {
            let v: Vec<&str> = target.split("@").collect();
            user = v[0];
            host = v[1];
        } else {
            host = target;
        }

        if host.contains(":") {
            let v: Vec<&str> = host.split(":").collect();
            host = v[0];
            port = v[1];
        }

        let tcp_stream = TcpStream::connect(format!("{}:{}", host, port)).map_err(|e| {
            ReployError::ConnectionFailed.with_context(format!(
                "Failed to connect to {}:{},Error: {}",
                host, port, e
            ))
        })?;
        self.session.set_tcp_stream(tcp_stream);
        self.session.handshake()?;

        if self.identity.exists() {
            self.session
                .userauth_pubkey_file(user, None, &self.identity, None)?;
        } else {
            return Err(ReployError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Identity file not found: {:?}", self.identity),
            )));
        }

        if !self.session.authenticated() {
            return Err(ReployError::AuthFailed);
        }

        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), ReployError> {
        self.session.disconnect(None, "connection closing", None)?;
        Ok(())
    }

    fn execute(&mut self, command: &str) -> Result<(), ReployError> {
        let mut channel = self.session.channel_session()?;
        channel.exec(command)?;

        let mut stdout = String::new();
        channel.read_to_string(&mut stdout)?;

        let mut stderr = String::new();
        channel.stderr().read_to_string(&mut stderr)?;

        self.stdio = Stdio {
            exit_code: channel.exit_status()?,
            stdout: stdout.trim().to_string(),
            stderr: stderr.trim().to_string(),
        };

        Ok(())
    }

    fn send(&self, source: &str, dest: &str) -> Result<(), ReployError> {
        let sftp = self.session.sftp()?;
        let local_path = Path::new(source);

        if !local_path.exists() {
            return Err(ReployError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Local file not found: {}", source),
            )));
        }

        if !local_path.is_file() {
            return Err(ReployError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Path is not a regular file: {}", source),
            )));
        }

        let file_reader = File::open(local_path).map_err(|e| {
            ReployError::Io(e).with_context(format!("Failed to open local file: {}", source))
        })?;

        let remote_path = Path::new(dest);
        let sftp_file = sftp.create(remote_path).map_err(|e| {
            ReployError::Ssh(e).with_context(format!("Failed to create remote file: {}", dest))
        })?;

        let mut file_reader = BufReader::with_capacity(BUF_SIZE, file_reader);
        let mut file_writer = BufWriter::with_capacity(BUF_SIZE, sftp_file);

        std::io::copy(&mut file_reader, &mut file_writer).map_err(|e| {
            ReployError::Io(e)
                .with_context(format!("Failed to copy data from {} to {}", source, dest))
        })?;

        Ok(())
    }

    fn recv(&self, source: &str, dest: &str) -> Result<(), ReployError> {
        let sftp = self.session.sftp()?;
        let remote_path = Path::new(source);
        let file_stat = sftp.stat(remote_path).map_err(|e| {
            ReployError::Ssh(e).with_context(format!("Failed to stat remote file: {}", source))
        })?;

        if !file_stat.is_file() {
            return Err(ReployError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Remote path is not a regular file: {}", source),
            )));
        }

        let remote_file = sftp.open(remote_path).map_err(|e| {
            ReployError::Ssh(e).with_context(format!("Failed to open remote file: {}", source))
        })?;

        let local_path = Path::new(dest);
        let local_file = File::create(local_path).map_err(|e| {
            ReployError::Io(e).with_context(format!("Failed to create local file: {}", dest))
        })?;

        let mut file_reader = BufReader::with_capacity(BUF_SIZE, remote_file);
        let mut file_writer = BufWriter::with_capacity(BUF_SIZE, local_file);

        std::io::copy(&mut file_reader, &mut file_writer).map_err(|e| {
            ReployError::Io(e)
                .with_context(format!("Failed to copy data from {} to {}", source, dest))
        })?;
        Ok(())
    }

    fn stdio(&self) -> &Stdio {
        &self.stdio
    }
}
