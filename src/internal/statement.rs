use internal::token::Token;

#[derive(Debug)]
pub struct Statement {
    pub token: Token,
    pub arguments: Vec<Token>,
}