use std::io;

use dialoguer::{theme::ColorfulTheme, Input, Password};
use regex::Regex;

use internal::error::ReployError;
use internal::executor::Executor;
use internal::token::Type;
use internal::*;

pub struct Evaluator {
    recipe: Recipe,
    is_end: bool,
    is_verbose: bool,
    executor: Box<dyn Executor>,
}

impl Evaluator {
    pub fn new(recipe: Recipe, verbose: bool, executor: Box<dyn Executor>) -> Self {
        Evaluator {
            recipe,
            is_end: false,
            is_verbose: verbose,
            executor,
        }
    }

    pub fn run(&mut self) -> Result<(), ReployError> {
        self.resolve_statement(self.recipe.task.to_vec())
    }

    fn resolve_statement(&mut self, statements: Vec<Statement>) -> Result<(), ReployError> {
        for statement in statements {
            if self.is_end {
                break;
            }

            match statement {
                Statement::Loop {
                    variable,
                    start,
                    end,
                    step,
                    body,
                } => {
                    if self.is_verbose {
                        println!("Executing LOOP statement");
                    }
                    self.resolve_loop(variable, start, end, step, body)?;
                }
                Statement::Simple { token, arguments } => {
                    let line_num = token.line_num;
                    if self.is_verbose {
                        println!(
                            "Line {}: executing statement: {:?}",
                            line_num, token.token_type
                        );
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
                            eprintln!(
                                "Line {}: unhandled statement type: {:?}",
                                line_num, token.token_type
                            );
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
        let cmd = self.replace_variable(arguments[0].literal.clone())?;
        if self.is_verbose {
            println!("run command: {}", cmd);
        }
        self.executor.execute(&cmd)?;
        Ok(())
    }

    fn resolve_let(&mut self, arguments: Vec<Token>) -> Result<(), ReployError> {
        match arguments[2].literal.as_str() {
            STDOUT => {
                self.recipe.variables.insert(
                    arguments[0].literal.clone(),
                    self.executor.stdio().stdout.clone(),
                );
            }
            STDERR => {
                self.recipe.variables.insert(
                    arguments[0].literal.clone(),
                    self.executor.stdio().stderr.clone(),
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
            println!("Sending: '{}' -> '{}'", source, dest);
        }

        self.executor.send(&source, &dest)
    }

    fn resolve_rcv(&self, arguments: Vec<Token>) -> Result<(), ReployError> {
        let source = self.replace_variable(arguments[0].literal.clone())?;
        let dest = self.replace_variable(arguments[1].literal.clone())?;

        if self.is_verbose {
            println!("Receiving: '{}' -> '{}'", source, dest);
        }

        self.executor.recv(&source, &dest)
    }

    fn resolve_when(&mut self, arguments: Vec<Token>) -> Result<(), ReployError> {
        let v1 = &arguments[0];
        let op = &arguments[1];
        let v2 = &arguments[2];
        let mut run_label;
        match v1.literal.as_str() {
            EXIT_CODE => run_label = self.executor.stdio().exit_code.to_string() == v2.literal,
            STDOUT => run_label = self.executor.stdio().stdout.contains(v2.literal.as_str()),
            STDERR => run_label = self.executor.stdio().stderr.contains(v2.literal.as_str()),
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
                    ReployError::Runtime(format!("Label {} not found", arguments[3].literal))
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
        let start_val = start
            .literal
            .parse::<i32>()
            .map_err(|_| ReployError::Runtime(format!("Invalid start value: {}", start.literal)))?;

        let end_val = end
            .literal
            .parse::<i32>()
            .map_err(|_| ReployError::Runtime(format!("Invalid end value: {}", end.literal)))?;

        let step_val = match step {
            Some(t) => t
                .literal
                .parse::<i32>()
                .map_err(|_| ReployError::Runtime(format!("Invalid step value: {}", t.literal)))?,
            None => 1,
        };

        let original_value = self.recipe.variables.get(&variable.literal).cloned();

        let mut current = start_val;
        while (step_val > 0 && current <= end_val) || (step_val < 0 && current >= end_val) {
            self.recipe
                .variables
                .insert(variable.literal.clone(), current.to_string());
            self.resolve_statement(body.clone())?;
            current += step_val;
        }

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
                    if std::net::TcpStream::connect(format!("127.0.0.1:{}", target)).is_ok() {
                        return Ok(());
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                Err(ReployError::Runtime(format!(
                    "Timeout waiting for port {} to open",
                    target
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
                    "Timeout waiting for file {} to exist",
                    target
                )))
            }
            _ => Err(ReployError::Runtime(format!(
                "Invalid wait mode: {}, expected 'port_open' or 'file_exists'",
                mode
            ))),
        }
    }

    fn resolve_target(&mut self, arguments: Vec<Token>) -> Result<(), ReployError> {
        let target = &arguments[0].literal;
        self.recipe
            .variables
            .insert(HOST_KEY.to_string(), target.clone());
        self.executor.connect(target)?;
        Ok(())
    }
}
