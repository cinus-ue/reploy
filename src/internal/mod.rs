use std::collections::HashMap;

pub mod evaluator;
pub mod parser;
pub mod lexer;
pub mod util;


#[derive(Debug)]
pub struct Recipe {
    pub statements: Vec<Statement>,
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

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: Type,
    pub literal: String,
}

#[derive(Clone, Debug)]
pub enum Type {
    EOF,
    SET,
    RUN,
    SND,
    RCV,
    SAY,
    END,
    WHEN,
    GOTO,
    TASK,
    LABEL,
    LBRACE,
    RBRACE,
    STRING,
    TARGET,
    UNKNOWN,
}

pub fn lookup_identifier(identifier: String) -> Type {
    return match identifier.trim() {
        "{" => Type::LBRACE,
        "}" => Type::RBRACE,
        "Set" => Type::SET,
        "Run" => Type::RUN,
        "Snd" => Type::SND,
        "Rcv" => Type::RCV,
        "Say" => Type::SAY,
        "End" => Type::END,
        "When" => Type::WHEN,
        "Goto" => Type::GOTO,
        "Task" => Type::TASK,
        "Label" => Type::LABEL,
        "Target" => Type::TARGET,
        _ => Type::UNKNOWN,
    };
}