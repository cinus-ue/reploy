use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};

use ssh2::{Channel, Sftp};

use internal::error::ReployError;
use internal::Stdio;

const BUF_SIZE: usize = 1024 * 1024;


pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

pub fn ssh_key() -> PathBuf {
    home_dir().map(|d| d.join(".ssh").join("id_rsa")).unwrap_or(PathBuf::new())
}

pub fn upload_file(local: &String, remote: &String, sftp: &Sftp) -> Result<(), ReployError> {
    let local_path = Path::new(local);
    
    if !local_path.exists() {
        return Err(ReployError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Local file not found: {}", local)
        )));
    }

    if !local_path.is_file() {
        return Err(ReployError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is not a regular file: {}", local)
        )));
    }

    let file_reader = File::open(local_path)
        .map_err(|e| ReployError::Io(e)
            .with_context(format!("Failed to open local file: {}", local)))?;

    let remote_path = Path::new(remote);
    let sftp_file = sftp.create(remote_path)
        .map_err(|e| ReployError::Ssh(e)
            .with_context(format!("Failed to create remote file: {}", remote)))?;

    let mut file_reader = BufReader::with_capacity(BUF_SIZE, file_reader);
    let mut file_writer = BufWriter::with_capacity(BUF_SIZE, sftp_file);
    
    std::io::copy(&mut file_reader, &mut file_writer)
        .map_err(|e| ReployError::Io(e)
            .with_context(format!("Failed to copy data from {} to {}", local, remote)))?;

    Ok(())
}

pub fn download_file(remote: &String, local: &String, sftp: &Sftp) -> Result<(), ReployError> {
    let remote_path = Path::new(remote);
    let file_stat = sftp.stat(remote_path)
        .map_err(|e| ReployError::Ssh(e)
            .with_context(format!("Failed to stat remote file: {}", remote)))?;

    if !file_stat.is_file() {
        return Err(ReployError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Remote path is not a regular file: {}", remote)
        )));
    }

    let remote_file = sftp.open(remote_path)
        .map_err(|e| ReployError::Ssh(e)
            .with_context(format!("Failed to open remote file: {}", remote)))?;

    let local_path = Path::new(local);
    let local_file = File::create(local_path)
        .map_err(|e| ReployError::Io(e)
            .with_context(format!("Failed to create local file: {}", local)))?;

    let mut file_reader = BufReader::with_capacity(BUF_SIZE, remote_file);
    let mut file_writer = BufWriter::with_capacity(BUF_SIZE, local_file);
    
    std::io::copy(&mut file_reader, &mut file_writer)
        .map_err(|e| ReployError::Io(e)
            .with_context(format!("Failed to copy data from {} to {}", remote, local)))?;

    Ok(())
}


pub fn consume_stdio(channel: &mut Channel) -> Stdio {
    let mut stdout = String::new();
    channel.read_to_string(&mut stdout).unwrap();

    let mut stderr = String::new();
    channel.stderr().read_to_string(&mut stderr).unwrap();

    return Stdio {
        exit_code: channel.exit_status().unwrap(),
        stdout: stdout.trim().to_string(),
        stderr: stderr.trim().to_string(),
    };
}