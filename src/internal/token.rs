#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: Type,
    pub line_num: usize,
    pub literal: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    EOF,
    SET,
    LET,
    RUN,
    SND,
    RCV,
    END,
    CALL,
    WHEN,
    TASK,
    ASK,
    PWD,
    LABEL,
    PRINT,
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
        "Let" => Type::LET,
        "Run" => Type::RUN,
        "Snd" => Type::SND,
        "Rcv" => Type::RCV,
        "End" => Type::END,
        "Call" => Type::CALL,
        "When" => Type::WHEN,
        "Task" => Type::TASK,
        "Ask" => Type::ASK,
        "Pwd" => Type::PWD,
        "Label" => Type::LABEL,
        "Print" => Type::PRINT,
        "Target" => Type::TARGET,
        _ => Type::UNKNOWN,
    };
}