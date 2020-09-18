use std::collections::LinkedList;

use internal::lexer::Lexer;
use internal::statement::Statement;
use internal::token::{Token, Type};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl Parser<'_> {
    pub fn new(lexer: Lexer) -> Parser {
        Parser { lexer }
    }

    pub fn parse(&mut self) -> LinkedList<Statement> {
        let mut result: LinkedList<Statement> = LinkedList::new();
        let mut sudo = false;
        let mut run = true;
        while run {
            let token = self.lexer.next_token();
            let mut arguments: Vec<Token> = Vec::new();
            match token.token_type {
                Type::RUN => {
                    arguments.push(self.lexer.next_token());
                    result.push_back(Statement { token, sudo, arguments });
                }
                Type::SUDO => {
                    sudo = true;
                }
                Type::SET | Type::UPLOAD | Type::DOWNLOAD => {
                    arguments.push(self.lexer.next_token());
                    arguments.push(self.lexer.next_token());
                    result.push_back(Statement { token, sudo, arguments });
                }
                Type::TARGET => {
                    arguments.push(self.lexer.next_token());
                    result.push_back(Statement { token, sudo, arguments });
                }
                _ => {
                    run = !self.lexer.is_eof();
                }
            }
        }
        return result;
    }
}

