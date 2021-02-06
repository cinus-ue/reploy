#[derive(Debug)]
pub struct Token {
    pub token_type: Type,
    pub literal: String,
}

#[derive(Debug)]
pub enum Type {
    EOF,
    STRING,
    RUN,
    SET,
    ECHO,
    CHECK,
    TARGET,
    UPLOAD,
    DOWNLOAD,
    UNKNOWN,
}

pub fn lookup_identifier(identifier: String) -> Type {
    return match identifier.trim() {
        "Run" => Type::RUN,
        "Set" => Type::SET,
        "Echo" => Type::ECHO,
        "Check" => Type::CHECK,
        "Target" => Type::TARGET,
        "Upload" => Type::UPLOAD,
        "Download" => Type::DOWNLOAD,
        _ => Type::UNKNOWN,
    };
}