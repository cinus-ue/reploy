use std::collections::HashMap;

use internal::{Recipe, Statement, Token, Type};
use internal::lexer::Lexer;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl Parser<'_> {
    pub fn new(lexer: Lexer) -> Parser {
        Parser { lexer }
    }

    pub fn parse(&mut self) -> Recipe {
        let mut recipe = Recipe {
            statements: Vec::new(),
            variables: HashMap::new(),
            labels: HashMap::new(),
        };
        loop {
            let token = self.lexer.next_token();
            match token.token_type {
                Type::TARGET => {
                    let mut arguments: Vec<Token> = Vec::new();
                    arguments.push(self.lexer.next_token());
                    recipe.statements.push(Statement { token, arguments });
                }
                Type::SET => {
                    let k = self.lexer.next_token();
                    let v = self.lexer.next_token();
                    recipe.variables.insert(k.literal, v.literal);
                }
                Type::TASK => {
                    recipe.statements.append(&mut self.parse_statement());
                }
                Type::LABEL => {
                    let label = self.lexer.next_token();
                    recipe.labels.insert(label.literal, self.parse_statement());
                }
                Type::EOF => {
                    break;
                }
                _ => {}
            }
        }
        recipe
    }

    fn parse_statement(&mut self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();
        loop {
            let token = self.lexer.next_token();
            let mut arguments: Vec<Token> = Vec::new();
            match token.token_type {
                Type::END => {
                    statements.push(Statement { token, arguments });
                }
                Type::RUN | Type::SAY => {
                    arguments.push(self.lexer.next_token());
                    statements.push(Statement { token, arguments });
                }
                Type::SND | Type::RCV => {
                    arguments.push(self.lexer.next_token());
                    arguments.push(self.lexer.next_token());
                    statements.push(Statement { token, arguments });
                }
                Type::WHEN => {
                    arguments.push(self.lexer.next_token());
                    arguments.push(self.lexer.next_token());
                    arguments.push(self.lexer.next_token());
                    arguments.push(self.lexer.next_token());
                    arguments.push(self.lexer.next_token());
                    statements.push(Statement { token, arguments });
                }
                Type::RBRACE => {
                    break;
                }
                _ => {}
            }
        }
        statements
    }
}

