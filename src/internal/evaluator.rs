use std::collections::HashMap;
use std::collections::LinkedList;
use std::net::TcpStream;
use std::path::PathBuf;

use regex::Regex;
use ssh2::Session;

use internal::statement::Statement;
use internal::status::Status;
use internal::token::Type;
use internal::util;

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
            identity: util::home_dir().map(|d| d.join(".ssh").join("id_rsa")).unwrap_or(PathBuf::new()),
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
                    Type::SET => {
                        self.resolve_set(statement)
                    }
                    Type::RUN => {
                        self.resolve_run(statement)
                    }
                    Type::ECHO => {
                        self.resolve_echo(statement)
                    }
                    Type::CHECK => {
                        self.resolve_check(statement)
                    }
                    Type::UPLOAD => {
                        self.resolve_upload(statement)
                    }
                    Type::DOWNLOAD => {
                        self.resolve_download(statement)
                    }
                    Type::TARGET => {
                        self.resolve_target(statement)
                    }
                    _ => eprintln!("unhandled statement: {:?}", statement)
                }
            });
        }
        if self.is_verbose {
            println!("Disconnecting from remote host");
        }
        assert!(self.ssh_session.disconnect(None, "connection closing", None).is_ok());
    }

    fn resolve_echo(&self, statement: Statement) {
        let v = &statement.arguments[0];
        println!("Reploy > {}", v.literal)
    }

    fn resolve_set(&mut self, statement: Statement) {
        let k = &statement.arguments[0];
        let v = &statement.arguments[1];
        self.variables.insert(k.literal.clone(), v.literal.clone());
    }

    fn resolve_run(&mut self, statement: Statement) {
        let mut channel = self.ssh_session.channel_session().unwrap();
        for v in statement.arguments.iter() {
            let cmd = self.replace_variable(v.literal.clone());
            if self.is_verbose {
                println!("run command: {}", cmd);
            }
            channel.exec(cmd.as_str()).expect("failed to run command");

            self.status = util::consume_stdio(&mut channel);
        }
    }

    fn resolve_check(&self, statement: Statement) {
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

    fn resolve_upload(&self, statement: Statement) {
        let s = &statement.arguments[0];
        let d = &statement.arguments[1];
        if self.is_verbose {
            println!("upload file:{}", s.literal);
        }
        let sftp = self.ssh_session.sftp().expect("SFTP error");
        util::upload_file(&s.literal, &d.literal, &sftp);
    }

    fn resolve_download(&self, statement: Statement) {
        let s = &statement.arguments[0];
        let d = &statement.arguments[1];
        if self.is_verbose {
            println!("download file:{}", s.literal);
        }
        let sftp = self.ssh_session.sftp().expect("SFTP error");
        util::download_file(&s.literal, &d.literal, &sftp);
    }
    fn resolve_target(&mut self, statement: Statement) {
        let target = &statement.arguments[0].literal;

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
}
