use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};

use ssh2::{Channel, Sftp};

use internal::Stdio;

const BUF_SIZE: usize = 1024 * 1024;


pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}


pub fn upload_file(local: &String, remote: &String, sftp: &Sftp) {
    let local_path = Path::new(local);
    if local_path.is_file() {
        let local_file = File::open(local_path).unwrap();
        let mut file_reader = BufReader::with_capacity(BUF_SIZE, local_file);
        let remote_file = sftp.create(Path::new(remote)).unwrap();
        let mut file_writer = BufWriter::with_capacity(BUF_SIZE, remote_file);
        io::copy(&mut file_reader, &mut file_writer).unwrap();
    }
}

pub fn download_file(remote: &String, local: &String, sftp: &Sftp) {
    let remote_path = Path::new(remote);
    match sftp.stat(remote_path) {
        Ok(f) => {
            if f.is_file() {
                let local_file = File::create(&Path::new(local)).unwrap();
                let mut file_writer = BufWriter::with_capacity(BUF_SIZE, local_file);
                let remote_file = sftp.open(remote_path).unwrap();
                let mut file_reader = BufReader::new(remote_file);
                io::copy(&mut file_reader, &mut file_writer).unwrap();
            }
        }
        Err(e) => panic!("failed to stat, {:?}", e)
    }
}


pub fn consume_stdio(channel: &mut Channel) -> Stdio {
    let mut stdout = String::new();
    channel.read_to_string(&mut stdout).unwrap();

    let mut stderr = String::new();
    channel.stderr().read_to_string(&mut stderr).unwrap();

    return Stdio {
        exit_code: channel.exit_status().unwrap(),
        stdout,
        stderr,
    };
}