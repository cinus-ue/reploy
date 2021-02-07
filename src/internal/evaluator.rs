use std::net::TcpStream;
use std::path::PathBuf;

use regex::Regex;
use ssh2::Session;

use internal::{Recipe, Statement};
use internal::Stdio;
use internal::Type;
use internal::util;

pub struct Evaluator {
    recipe: Recipe,
    is_stop: bool,
    is_verbose: bool,
    identity: PathBuf,
    ssh_session: Session,
    stdio: Stdio,
}


impl Evaluator {
    pub fn new(recipe: Recipe, verbose: bool) -> Evaluator {
        Evaluator {
            recipe,
            is_stop: false,
            is_verbose: verbose,
            identity: util::home_dir().map(|d| d.join(".ssh").join("id_rsa")).unwrap_or(PathBuf::new()),
            ssh_session: Session::new().unwrap(),
            stdio: Stdio { exit_code: 0, stdout: String::new(), stderr: String::new() },
        }
    }

    pub fn set_identity(&mut self, identity: &str) {
        self.identity = PathBuf::from(identity);
    }

    pub fn run(&mut self) {
        self.resolve_statement(self.recipe.statements.to_vec());
        assert!(self.ssh_session.disconnect(None, "connection closing", None).is_ok());
    }

    fn resolve_statement(&mut self, statements: Vec<Statement>) {
        for statement in statements {
            if self.is_stop {
                break;
            }
            match statement.token.token_type {
                Type::TARGET => {
                    self.resolve_target(statement)
                }
                Type::RUN => {
                    self.resolve_run(statement)
                }
                Type::ECHO => {
                    self.resolve_echo(statement)
                }
                Type::WHEN => {
                    self.resolve_when(statement)
                }
                Type::UPLOAD => {
                    self.resolve_upload(statement)
                }
                Type::DOWNLOAD => {
                    self.resolve_download(statement)
                }
                Type::EXIT => {
                    self.is_stop = true;
                }
                _ => eprintln!("unhandled statement: {:?}", statement)
            }
        }
    }

    fn resolve_run(&mut self, statement: Statement) {
        let mut channel = self.ssh_session.channel_session().unwrap();
        let cmd = self.replace_variable(statement.arguments[0].literal.clone());
        if self.is_verbose {
            println!("run command: {}", cmd);
        }
        channel.exec(cmd.as_str()).expect("failed to run command");
        self.stdio = util::consume_stdio(&mut channel);
        if self.is_verbose {
            println!("stdio: {:?}", self.stdio);
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

    fn resolve_when(&mut self, statement: Statement) {
        let v1 = &statement.arguments[0];
        let op = &statement.arguments[1];
        let v2 = &statement.arguments[2];
        let mut run_label = false;
        match v1.literal.as_str() {
            "exit_code" => {
                match op.literal.as_str() {
                    "==" => { run_label = self.stdio.exit_code.to_string() == v2.literal }
                    "!=" => { run_label = self.stdio.exit_code.to_string() != v2.literal }
                    _ => {}
                }
            }
            "stdout" => {
                match op.literal.as_str() {
                    "==" => { run_label = self.stdio.stdout.contains(v2.literal.as_str()) }
                    "!=" => { run_label = !self.stdio.stdout.contains(v2.literal.as_str()) }
                    _ => {}
                }
            }
            "stderr" => {
                match op.literal.as_str() {
                    "==" => { run_label = self.stdio.stderr.contains(v2.literal.as_str()) }
                    "!=" => { run_label = !self.stdio.stderr.contains(v2.literal.as_str()) }
                    _ => {}
                }
            }
            _ => {}
        }
        if run_label {
            self.resolve_statement(self.recipe.labels.get(statement.arguments[4].literal.as_str()).unwrap().to_vec())
        }
    }

    fn resolve_echo(&self, statement: Statement) {
        let v = &statement.arguments[0];
        println!("Reploy > {}", v.literal)
    }

    fn replace_variable(&self, mut cmd: String) -> String {
        for cap in Regex::new(r"\$\{(.*?)}").unwrap().captures_iter(&cmd.clone()) {
            let var = cap.get(0).unwrap().as_str();
            let key = var.trim_start_matches("${").trim_end_matches("}");
            if self.recipe.variables.contains_key(key) {
                cmd = cmd.replace(var, self.recipe.variables.get(key).unwrap().as_str());
            }
        }
        cmd
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
        } else {
            panic!("File does not exist: {:?}", self.identity)
        }
        assert!(self.ssh_session.authenticated());
    }
}
