use std::net::TcpStream;
use std::path::PathBuf;

use regex::Regex;
use ssh2::Session;

use internal::*;
use internal::token::Type;

pub struct Evaluator {
    recipe: Recipe,
    is_end: bool,
    is_verbose: bool,
    identity: PathBuf,
    ssh_session: Session,
    ssh_stdio: Stdio,
}

impl Evaluator {
    pub fn new(recipe: Recipe, verbose: bool) -> Evaluator {
        Evaluator {
            recipe,
            is_end: false,
            is_verbose: verbose,
            identity: util::ssh_key(),
            ssh_session: Session::new().unwrap(),
            ssh_stdio: Stdio { exit_code: 0, stdout: String::new(), stderr: String::new() },
        }
    }

    pub fn set_identity(&mut self, identity: &str) {
        self.identity = PathBuf::from(identity);
    }

    pub fn run(&mut self) {
        self.resolve_statement(self.recipe.task.to_vec());
        assert!(self.ssh_session.disconnect(None, "connection closing", None).is_ok());
    }

    fn resolve_statement(&mut self, statements: Vec<Statement>) {
        for statement in statements {
            if self.is_end {
                break;
            }
            if self.is_verbose {
                println!("resolve statement: {:?}", statement)
            }
            match statement.token.token_type {
                Type::TARGET => {
                    self.resolve_target(statement)
                }
                Type::PRINT => {
                    self.resolve_print(statement)
                }
                Type::RUN => {
                    self.resolve_run(statement)
                }
                Type::LET => {
                    self.resolve_let(statement)
                }
                Type::WHEN => {
                    self.resolve_when(statement)
                }
                Type::SND => {
                    self.resolve_snd(statement)
                }
                Type::RCV => {
                    self.resolve_rcv(statement)
                }
                Type::CALL => {
                    self.resolve_call(statement)
                }
                Type::END => {
                    self.is_end = true;
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
        self.ssh_stdio = util::consume_stdio(&mut channel);
    }

    fn resolve_let(&mut self, statement: Statement) {
        match statement.arguments[2].literal.as_str() {
            STDOUT => {
                self.recipe.variables.insert(statement.arguments[0].literal.clone(), self.ssh_stdio.stdout.clone());
            }
            STDERR => {
                self.recipe.variables.insert(statement.arguments[0].literal.clone(), self.ssh_stdio.stderr.clone());
            }
            _ => {}
        }
    }

    fn resolve_snd(&self, statement: Statement) {
        let s = self.replace_variable(statement.arguments[0].literal.clone());
        let d = self.replace_variable(statement.arguments[1].literal.clone());
        if self.is_verbose {
            println!("upload file:{}", s);
        }
        let sftp = self.ssh_session.sftp().expect("SFTP error");
        util::upload_file(&s, &d, &sftp);
    }

    fn resolve_rcv(&self, statement: Statement) {
        let s = self.replace_variable(statement.arguments[0].literal.clone());
        let d = self.replace_variable(statement.arguments[1].literal.clone());
        if self.is_verbose {
            println!("download file:{}", s);
        }
        let sftp = self.ssh_session.sftp().expect("SFTP error");
        util::download_file(&s, &d, &sftp);
    }

    fn resolve_when(&mut self, statement: Statement) {
        let v1 = &statement.arguments[0];
        let op = &statement.arguments[1];
        let v2 = &statement.arguments[2];
        let mut run_label = false;
        match v1.literal.as_str() {
            EXIT_CODE => {
                run_label = self.ssh_stdio.exit_code.to_string() == v2.literal
            }
            STDOUT => {
                run_label = self.ssh_stdio.stdout.contains(v2.literal.as_str())
            }
            STDERR => {
                run_label = self.ssh_stdio.stderr.contains(v2.literal.as_str())
            }
            _ => {}
        }
        if op.literal.as_str() != EQEQ {
            run_label = !run_label;
        }
        if run_label {
            self.resolve_statement(self.recipe.labels.get(statement.arguments[4].literal.as_str()).unwrap().to_vec())
        }
    }

    fn resolve_call(&mut self, statement: Statement) {
        self.resolve_statement(self.recipe.labels.get(statement.arguments[0].literal.as_str()).unwrap().to_vec())
    }

    fn resolve_print(&self, statement: Statement) {
        println!("Reploy > {}", self.replace_variable(statement.arguments[0].literal.clone()))
    }

    fn replace_variable(&self, mut s: String) -> String {
        for cap in Regex::new(r"\{\{(.*?)}}").unwrap().captures_iter(&s.clone()) {
            let var = cap.get(0).unwrap().as_str();
            let key = var.trim_start_matches("{{").trim_end_matches("}}");
            if self.recipe.variables.contains_key(key) {
                s = s.replace(var, self.recipe.variables.get(key).unwrap().as_str());
            }
        }
        s
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
            panic!("file does not exist: {:?}", self.identity)
        }
        assert!(self.ssh_session.authenticated());
    }
}
