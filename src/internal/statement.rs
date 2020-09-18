use internal::token::Token;

#[derive(Debug)]
pub struct Statement {
    pub token: Token,
    pub sudo: bool,
    pub arguments: Vec<Token>,
}