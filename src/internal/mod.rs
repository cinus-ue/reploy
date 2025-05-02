use std::collections::HashMap;

use token::Token;

pub mod error;
pub mod evaluator;
pub mod executor;
pub mod lexer;
pub mod parser;
mod token;
mod util;

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
        branches: Vec<(Token, Vec<Statement>)>,
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
