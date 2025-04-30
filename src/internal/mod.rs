use std::collections::HashMap;

use internal::token::Token;

pub mod evaluator;
pub mod parser;
pub mod lexer;
pub mod error;
mod util;
mod token;

const HOST_KEY: &str = "$HOST";

const EQEQ: &str = "==";
const STDOUT: &str = "stdout";
const STDERR: &str = "stderr";
const EXIT_CODE: &str = "exit_code";

#[derive(Debug)]
pub struct Recipe {
    pub task: Vec<Statement>,
    pub variables: HashMap<String, String>,
    pub labels: HashMap<String, Vec<Statement>>,
}

#[derive(Clone, Debug)]
pub struct Statement {
    pub token: Token,
    pub arguments: Vec<Token>,
}

#[derive(Debug)]
pub struct Stdio {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

