use std::collections::HashMap;
use std::collections::LinkedList;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter};
use std::io::Read;
use std::net::TcpStream;
use std::path::{Path, PathBuf};

use regex::Regex;
use ssh2::{Channel, Session, Sftp};

use internal::statement::Statement;
use internal::status::Status;
use internal::token::Type;
use internal::util;

const BUF_SIZE: usize = 1024 * 1024;

pub struct Evaluator {
    statements: LinkedList<Statement>,
    identity: PathBuf,
    variables: HashMap<String, String>,
    is_verbose: bool,
    ssh_session: Session,
    status: Status,
}


impl Evaluator {
    pub fn new(statements: LinkedList<Statement>, verbose: bool) -> Evaluator {
        Evaluator {
            statements,
            identity: util::home_dir().map(|d| d.join(".ssh")
                .join("id_rsa")).unwrap_or(PathBuf::new()),
            variables: HashMap::new(),
            is_verbose: verbose,
            ssh_session: Session::new().unwrap(),
            status: Status { exit_code: 0, stdout: String::new(), stderr: String::new() },
        }
    }

    pub fn set_identity(&mut self, identity: &str) {
        self.identity = PathBuf::from(identity);
    }

    pub fn run(&mut self) {
        while !self.statements.is_empty() {
            self.statements.pop_front().map(|statement| {
                match statement.token.token_type {
                    Type::RUN => {
                        let mut channel = self.ssh_session.channel_session().unwrap();
                        for t in statement.arguments.iter() {
                            let cmd = self.replace_variable(t.literal.clone());
                            if self.is_verbose {
                                println!("run command: {}", cmd);
                            }
                            channel.exec(cmd.as_str()).expect("failed to run command");
                            self.status = consume_stdio(&mut channel);
                        }
                    }
                    Type::ASSERT => {
                        let t = &statement.arguments[0];
                        let v = &statement.arguments[1];
                        match t.literal.as_str() {
                            "exit_code" => {
                                if self.status.exit_code.to_string() != v.literal {
                                    panic!("assertion failed:exit_code {}, expected {}", self.status.exit_code, v.literal)
                                }
                            }
                            "stdout" => {
                                if !(self.status.stdout.contains(v.literal.as_str())) {
                                    panic!("assertion failed:stdout {}", self.status.stdout)
                                }
                            }
                            "stderr" => {
                                if !(self.status.stderr.contains(v.literal.as_str())) {
                                    panic!("assertion failed:stderr {}", self.status.stderr)
                                }
                            }
                            _ => {}
                        }
                    }
                    Type::SET => {
                        let k = &statement.arguments[0];
                        let v = &statement.arguments[1];
                        self.variables.insert(k.literal.clone(), v.literal.clone());
                    }
                    Type::UPLOAD => {
                        let s = &statement.arguments[0];
                        let d = &statement.arguments[1];
                        if self.is_verbose {
                            println!("upload file:{}", s.literal);
                        }
                        let sftp = self.ssh_session.sftp().expect("failed to upload file");
                        upload_file(&s.literal, &d.literal, &sftp);
                    }
                    Type::DOWNLOAD => {
                        let s = &statement.arguments[0];
                        let d = &statement.arguments[1];
                        if self.is_verbose {
                            println!("download file:{}", s.literal);
                        }
                        let sftp = self.ssh_session.sftp().expect("failed to download file");
                        download_file(&s.literal, &d.literal, &sftp);
                    }
                    Type::TARGET => {
                        match statement.arguments.get(0) {
                            Some(t) => {
                                self.connect_to(&t.literal);
                            }
                            None => panic!("failed to connect to target")
                        }
                    }
                    _ => eprintln!("unhandled statement: {:?}", statement)
                }
            });
        }
        if self.is_verbose {
            println!("Disconnecting from remote-host");
        }
        assert!(self.ssh_session.disconnect(None, "connection closing", None).is_ok());
    }


    fn replace_variable(&self, mut cmd: String) -> String {
        for cap in Regex::new(r"\$\{(.*?)}").unwrap().captures_iter(&cmd.clone()) {
            let var = cap.get(0).unwrap().as_str();
            let key = var.trim_start_matches("${").trim_end_matches("}");
            if self.variables.contains_key(key) {
                cmd = cmd.replace(var, self.variables.get(key).unwrap().as_str());
            }
        }
        cmd
    }

    fn connect_to(&mut self, target: &String) {
        let mut user = "root";
        let mut port = "22";
        let mut host;

        if target.contains("@") {
            let v: Vec<&str> = target.split("@").collect();
            user = v[0];
            host = v[1];
        } else {
            host = target.as_str();
        }
        if host.contains(":") {
            let v: Vec<&str> = host.split(":").collect();
            host = v[0];
            port = v[1];
        }
        if self.is_verbose {
            println!("user:{}, host:{}, port:{}", user, host, port);
            println!("identity: {:?}", self.identity);
        }
        self.ssh_session.set_tcp_stream(TcpStream::connect(format!("{}:{}", host, port)).expect("failed to connect to target"));
        self.ssh_session.handshake().expect("handshake failed");
        if self.identity.exists() {
            self.ssh_session
                .userauth_pubkey_file(user, None, self.identity.as_path(), None)
                .expect("authentication failed");
        }
        assert!(self.ssh_session.authenticated());
    }
}

fn upload_file(local: &String, remote: &String, sftp: &Sftp) {
    let local_path = Path::new(local);
    if local_path.is_file() {
        let local_file = File::open(local_path).unwrap();
        let mut file_reader = BufReader::with_capacity(BUF_SIZE, local_file);
        let remote_file = sftp.create(Path::new(remote)).unwrap();
        let mut file_writer = BufWriter::with_capacity(BUF_SIZE, remote_file);
        io::copy(&mut file_reader, &mut file_writer).unwrap();
    }
}

fn download_file(remote: &String, local: &String, sftp: &Sftp) {
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


fn consume_stdio(channel: &mut Channel) -> Status {
    let mut stdout = String::new();
    channel.read_to_string(&mut stdout).unwrap();

    let mut stderr = String::new();
    channel.stderr().read_to_string(&mut stderr).unwrap();

    if !stdout.is_empty() {
        println!("stdout: {}", stdout.trim());
    }

    if !stderr.is_empty() {
        println!("stderr: {}", stderr.trim());
    }
    return Status {
        exit_code: channel.exit_status().unwrap(),
        stdout,
        stderr,
    };
}