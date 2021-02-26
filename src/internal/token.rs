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
    END,
    WHEN,
    GOTO,
    TASK,
    LABEL,
    LBRACE,
    RBRACE,
    STRING,
    TARGET,
    COMMENT,
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
        "End" => Type::END,
        "When" => Type::WHEN,
        "Goto" => Type::GOTO,
        "Task" => Type::TASK,
        "Label" => Type::LABEL,
        "Comment" => Type::COMMENT,
        "Target" => Type::TARGET,
        _ => Type::UNKNOWN,
    };
}