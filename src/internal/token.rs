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
    SUDO,
    TARGET,
    UPLOAD,
    DOWNLOAD,
    UNKNOWN,
}

pub fn lookup_identifier(identifier: String) -> Type {
    return match identifier.trim() {
        "Run" => Type::RUN,
        "Set" => Type::SET,
        "Sudo" => Type::SUDO,
        "Target" => Type::TARGET,
        "Upload" => Type::UPLOAD,
        "Download" => Type::DOWNLOAD,
        _ => Type::UNKNOWN,
    };
}