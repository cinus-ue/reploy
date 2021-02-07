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
    RUN,
    SET,
    ECHO,
    EXIT,
    WHEN,
    GOTO,
    TASK,
    LABEL,
    LBRACE,
    RBRACE,
    STRING,
    TARGET,
    UPLOAD,
    DOWNLOAD,
    UNKNOWN,
}

pub fn lookup_identifier(identifier: String) -> Type {
    return match identifier.trim() {
        "{" => Type::LBRACE,
        "}" => Type::RBRACE,
        "Run" => Type::RUN,
        "Set" => Type::SET,
        "Echo" => Type::ECHO,
        "Exit" => Type::EXIT,
        "When" => Type::WHEN,
        "Goto" => Type::GOTO,
        "Task" => Type::TASK,
        "Label" => Type::LABEL,
        "Target" => Type::TARGET,
        "Upload" => Type::UPLOAD,
        "Download" => Type::DOWNLOAD,
        _ => Type::UNKNOWN,
    };
}