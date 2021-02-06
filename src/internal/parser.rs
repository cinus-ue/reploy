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
        let mut statements: LinkedList<Statement> = LinkedList::new();
        let mut is_end = false;
        while !is_end {
            let token = self.lexer.next_token();
            let mut arguments: Vec<Token> = Vec::new();
            match token.token_type {
                Type::RUN | Type::ECHO | Type::TARGET => {
                    arguments.push(self.lexer.next_token());
                    statements.push_back(Statement { token, arguments });
                }
                Type::SET | Type::CHECK | Type::UPLOAD | Type::DOWNLOAD => {
                    arguments.push(self.lexer.next_token());
                    arguments.push(self.lexer.next_token());
                    statements.push_back(Statement { token, arguments });
                }
                _ => {
                    is_end = self.lexer.is_eof();
                }
            }
        }
        statements
    }
}

