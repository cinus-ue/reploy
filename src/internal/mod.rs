use std::collections::HashMap;

use internal::token::Token;

pub mod error;
pub mod evaluator;
pub mod executor;
pub mod lexer;
pub mod parser;
mod token;
mod util;

const HOST_KEY: &str = "$HOST";

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
pub enum Statement {
    Simple {
        token: Token,
        arguments: Vec<Token>,
    },
    Loop {
        variable: Token,
        start: Token,
        end: Token,
        step: Option<Token>,
        body: Vec<Statement>,
    },
    ListLoop {
        variable: Token,
        list: Token,
        body: Vec<Statement>,
    },
    While {
        condition: Token,
        body: Vec<Statement>,
    },
    When {
        condition: Token,
        body: Vec<Statement>,
    },
}

#[derive(Debug)]
pub struct Stdio {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Default for Stdio {
    fn default() -> Self {
        Self {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}
