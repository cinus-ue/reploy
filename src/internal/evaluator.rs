use std::io;
use std::net::TcpStream;
use std::path::PathBuf;

use dialoguer::{theme::ColorfulTheme, Input, Password};
use regex::Regex;
use ssh2::Session;

use internal::token::Type;
use internal::*;

use internal::error::ReployError;

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
            ssh_stdio: Stdio {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            },
        }
    }

    pub fn set_identity(&mut self, identity: &str) {
        self.identity = PathBuf::from(identity);
    }

    pub fn run(&mut self) -> Result<(), ReployError> {
        self.resolve_statement(self.recipe.task.to_vec())?;
        self.ssh_session
            .disconnect(None, "connection closing", None)?;
        Ok(())
    }

    fn resolve_statement(&mut self, statements: Vec<Statement>) -> Result<(), ReployError> {
        for statement in statements {
            if self.is_end {
                break;
            }
            
            match statement {
                Statement::Loop { variable, start, end, step, body } => {
                    if self.is_verbose {
                        println!("Executing LOOP statement");
                    }
                    self.resolve_loop(variable, start, end, step, body)?;
                }
                Statement::Simple { token, arguments } => {
                    let line_num = token.line_num;
                    if self.is_verbose {
                        println!("Line {}: executing statement: {:?}", line_num, token.token_type);
                    }
                    let result = match token.token_type {
                        Type::TARGET => self.resolve_target(arguments),
                        Type::PRINT => self.resolve_print(arguments),
                        Type::RUN => self.resolve_run(arguments),
                        Type::LET => self.resolve_let(arguments),
                        Type::ASK => self.resolve_ask(arguments),
                        Type::PWD => self.resolve_password(arguments),
                        Type::WHEN => self.resolve_when(arguments),
                        Type::SND => self.resolve_snd(arguments),
                        Type::RCV => self.resolve_rcv(arguments),
                        Type::CALL => self.resolve_call(arguments),
                        Type::WAIT => self.resolve_wait(arguments),
                        Type::END => {
                            self.is_end = true;
                            Ok(())
                        }
                        _ => {
                            eprintln!("Line {}: unhandled statement type: {:?}", line_num, token.token_type);
                            Ok(())
                        }
                    };
                    result.map_err(|e| ReployError::Runtime(format!("Line {}: {}", line_num, e)))?
                }
            }
        }
        Ok(())
    }

    fn resolve_run(&mut self, arguments: Vec<Token>) -> Result<(), ReployError> {
        let mut channel = self.ssh_session.channel_session()?;
        let cmd = self.replace_variable(arguments[0].literal.clone())?;
        if self.is_verbose {
            println!("run command: {}", cmd);
        }
        channel.exec(cmd.as_str()).map_err(|e| {
            ReployError::CommandFailed(
                -1,
                format!("Failed to execute command: {},Error: {}", cmd, e),
            )
        })?;
        self.ssh_stdio = util::consume_stdio(&mut channel);
        Ok(())
    }

    fn resolve_let(&mut self, arguments: Vec<Token>) -> Result<(), ReployError> {
        match arguments[2].literal.as_str() {
            STDOUT => {
                self.recipe.variables.insert(
                    arguments[0].literal.clone(),
                    self.ssh_stdio.stdout.clone(),
                );
            }
            STDERR => {
                self.recipe.variables.insert(
                    arguments[0].literal.clone(),
                    self.ssh_stdio.stderr.clone(),
                );
            }
            _ => {
                return Err(ReployError::Runtime(format!(
                    "Invalid LET operation: {}",
                    arguments[2].literal
                )));
            }
        }
        Ok(())
    }

    fn resolve_ask(&mut self, arguments: Vec<Token>) -> Result<(), ReployError> {
        let input = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(arguments[0].literal.clone())
            .interact_text()
            .map_err(|e| {
                ReployError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to read input: {}", e),
                ))
            })?;
        self.recipe
            .variables
            .insert(arguments[1].literal.clone(), input);
        Ok(())
    }

    fn resolve_password(&mut self, arguments: Vec<Token>) -> Result<(), ReployError> {
        let password = Password::with_theme(&ColorfulTheme::default())
            .with_prompt(arguments[0].literal.clone())
            // .with_confirmation("Repeat password", "Error: the passwords don't match.")
            .interact()
            .map_err(|e| {
                ReployError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to read password: {}", e),
                ))
            })?;
        self.recipe
            .variables
            .insert(arguments[1].literal.clone(), password);
        Ok(())
    }

    fn resolve_snd(&self, arguments: Vec<Token>) -> Result<(), ReployError> {
        let source = self.replace_variable(arguments[0].literal.clone())?;
        let dest = self.replace_variable(arguments[1].literal.clone())?;

        if self.is_verbose {
            println!("[SFTP] Uploading: '{}' -> '{}'", source, dest);
        }

        let sftp = self.ssh_session.sftp().map_err(|e| {
            ReployError::Ssh(e).with_context(format!(
                "SFTP session initialization failed for upload operation"
            ))
        })?;

        util::upload_file(&source, &dest, &sftp).map_err(|e| match e {
            ReployError::Io(io_err) => ReployError::Io(io_err).with_context(format!(
                "File upload failed | Source: '{}' | Destination: '{}'",
                source, dest
            )),
            ReployError::Ssh(ssh_err) => ReployError::Ssh(ssh_err).with_context(format!(
                "SFTP upload failed | Source: '{}' | Destination: '{}'",
                source, dest
            )),
            other => other,
        })
    }

    fn resolve_rcv(&self, arguments: Vec<Token>) -> Result<(), ReployError> {
        let source = self.replace_variable(arguments[0].literal.clone())?;
        let dest = self.replace_variable(arguments[1].literal.clone())?;

        if self.is_verbose {
            println!("[SFTP] Downloading: '{}' -> '{}'", source, dest);
        }

        let sftp = self.ssh_session.sftp().map_err(|e| {
            ReployError::Ssh(e).with_context(format!(
                "SFTP session initialization failed for download operation"
            ))
        })?;

        util::download_file(&source, &dest, &sftp).map_err(|e| match e {
            ReployError::Io(io_err) => ReployError::Io(io_err).with_context(format!(
                "File download failed | Source: '{}' | Destination: '{}'",
                source, dest
            )),
            ReployError::Ssh(ssh_err) => ReployError::Ssh(ssh_err).with_context(format!(
                "SFTP download failed | Source: '{}' | Destination: '{}'",
                source, dest
            )),
            other => other,
        })
    }

    fn resolve_when(&mut self, arguments: Vec<Token>) -> Result<(), ReployError> {
        let v1 = &arguments[0];
        let op = &arguments[1];
        let v2 = &arguments[2];
        let mut run_label;
        match v1.literal.as_str() {
            EXIT_CODE => run_label = self.ssh_stdio.exit_code.to_string() == v2.literal,
            STDOUT => run_label = self.ssh_stdio.stdout.contains(v2.literal.as_str()),
            STDERR => run_label = self.ssh_stdio.stderr.contains(v2.literal.as_str()),
            _ => {
                let var_value =
                    self.recipe
                        .variables
                        .get(v1.literal.as_str())
                        .ok_or_else(|| {
                            ReployError::Runtime(format!("Variable {} not found", v1.literal))
                        })?;
                run_label = var_value.contains(v2.literal.as_str())
            }
        }
        if op.literal.as_str() != EQEQ {
            run_label = !run_label;
        }
        if run_label {
        let label_statements = self
            .recipe
            .labels
            .get(arguments[3].literal.as_str())
                .ok_or_else(|| {
                    ReployError::Runtime(format!(
                        "Label {} not found",
                        arguments[3].literal
                    ))
                })?;
            self.resolve_statement(label_statements.to_vec())?;
        }
        Ok(())
    }

    fn resolve_call(&mut self, arguments: Vec<Token>) -> Result<(), ReployError> {
        let mut label = arguments[0].literal.clone();
        if label.starts_with("{{") && label.ends_with("}}") {
            label = self.replace_variable(label)?
        }
        let label_statements = self
            .recipe
            .labels
            .get(label.as_str())
            .ok_or_else(|| ReployError::Runtime(format!("Label {} not found", label)))?;
        self.resolve_statement(label_statements.to_vec())
    }

    fn resolve_print(&self, arguments: Vec<Token>) -> Result<(), ReployError> {
        let host = self
            .recipe
            .variables
            .get(HOST_KEY)
            .ok_or_else(|| ReployError::Runtime("HOST_KEY not found".to_string()))?;
        let message = self.replace_variable(arguments[0].literal.clone())?;
        println!("{} > {}", host, message);
        Ok(())
    }

    fn replace_variable(&self, mut s: String) -> Result<String, ReployError> {
        let re = Regex::new(r"\{\{(.*?)}}")
            .map_err(|e| ReployError::Runtime(format!("Failed to compile regex: {}", e)))?;

        for cap in re.captures_iter(&s.clone()) {
            let var = cap
                .get(0)
                .ok_or_else(|| ReployError::Runtime("Invalid variable pattern".to_string()))?
                .as_str();
            let key = var.trim_start_matches("{{").trim_end_matches("}}");
            if let Some(value) = self.recipe.variables.get(key) {
                s = s.replace(var, value.as_str());
            }
        }
        Ok(s)
    }

    fn resolve_loop(
        &mut self,
        variable: Token,
        start: Token,
        end: Token,
        step: Option<Token>,
        body: Vec<Statement>,
    ) -> Result<(), ReployError> {
        // Parse start value
        let start_val = start.literal.parse::<i32>().map_err(|_| 
            ReployError::Runtime(format!("Invalid start value: {}", start.literal))
        )?;
        
        // Parse end value
        let end_val = end.literal.parse::<i32>().map_err(|_| 
            ReployError::Runtime(format!("Invalid end value: {}", end.literal))
        )?;
        
        // Parse step value (default to 1 if not provided)
        let step_val = match step {
            Some(t) => t.literal.parse::<i32>().map_err(|_| 
                ReployError::Runtime(format!("Invalid step value: {}", t.literal))
            )?,
            None => 1,
        };

        // Save original variable value if it exists
        let original_value = self.recipe.variables.get(&variable.literal).cloned();

        // Execute loop
        let mut current = start_val;
        while (step_val > 0 && current <= end_val) || (step_val < 0 && current >= end_val) {
            // Set loop variable
            self.recipe.variables.insert(variable.literal.clone(), current.to_string());
            
            // Execute loop body
            self.resolve_statement(body.clone())?;
            
            // Update loop variable
            current += step_val;
        }

        // Restore original variable value if it existed
        if let Some(val) = original_value {
            self.recipe.variables.insert(variable.literal.clone(), val);
        } else {
            self.recipe.variables.remove(&variable.literal);
        }

        Ok(())
    }

    fn resolve_wait(&mut self, arguments: Vec<Token>) -> Result<(), ReployError> {
        let mode = self.replace_variable(arguments[0].literal.clone())?;
        let target = self.replace_variable(arguments[1].literal.clone())?;
        let timeout = arguments[2].literal.parse::<u64>().unwrap_or(30);
        
        let start = std::time::Instant::now();
        
        match mode.as_str() {
            "port_open" => {
                while start.elapsed().as_secs() < timeout {
                    if TcpStream::connect(format!("127.0.0.1:{}", target)).is_ok() {
                        return Ok(());
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                Err(ReployError::Runtime(format!(
                    "Timeout waiting for port {} to open", target
                )))
            }
            "file_exists" => {
                while start.elapsed().as_secs() < timeout {
                    if std::path::Path::new(&target).exists() {
                        return Ok(());
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                Err(ReployError::Runtime(format!(
                    "Timeout waiting for file {} to exist", target
                )))
            }
            _ => Err(ReployError::Runtime(format!(
                "Invalid wait mode: {}, expected 'port_open' or 'file_exists'", mode
            ))),
        }
    }

    fn resolve_target(&mut self, arguments: Vec<Token>) -> Result<(), ReployError> {
        
        let target = &arguments[0].literal;
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
        self.recipe
            .variables
            .insert(HOST_KEY.to_string(), host.to_string());
        if self.is_verbose {
            println!("user:{}, host:{}, port:{}", user, host, port);
            println!("identity: {:?}", self.identity);
        }

        let tcp_stream = TcpStream::connect(format!("{}:{}", host, port)).map_err(|e| {
            ReployError::ConnectionFailed.with_context(format!(
                "Failed to connect to {}:{},Error: {}",
                host, port, e
            ))
        })?;

        self.ssh_session.set_tcp_stream(tcp_stream);
        self.ssh_session.handshake()?;

        if self.identity.exists() {
            self.ssh_session
                .userauth_pubkey_file(user, None, self.identity.as_path(), None)
                .map_err(|_| ReployError::AuthFailed)?;
        } else {
            return Err(ReployError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Identity file not found: {:?}", self.identity),
            )));
        }

        if !self.ssh_session.authenticated() {
            return Err(ReployError::AuthFailed);
        }

        Ok(())
    }
}
