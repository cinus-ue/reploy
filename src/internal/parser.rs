use std::collections::HashMap;

use internal::{Recipe, Statement};
use internal::lexer::Lexer;
use internal::token::{Token, Type};

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        Parser { lexer }
    }

    pub fn parse(&mut self) -> Recipe {
        let mut recipe = Recipe {
            task: Vec::new(),
            variables: HashMap::new(),
            labels: HashMap::new(),
        };
        loop {
            let token = self.lexer.next_token();
            match token.token_type {
                Type::TARGET => {
                    let mut arguments: Vec<Token> = Vec::new();
                    arguments.push(self.lexer.next_token());
                    recipe.task.push(Statement { token, arguments });
                }
                Type::SET => {
                    let k = self.lexer.next_token();
                    let v = self.lexer.next_token();
                    recipe.variables.insert(k.literal, v.literal);
                }
                Type::TASK => {
                    recipe.task.append(&mut self.parse_statement());
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
            let mut len = 0;
            match token.token_type {
                Type::RUN | Type::PRINT | Type::CALL => {
                    len = 1;
                }
                Type::SND | Type::RCV | Type::ASK | Type::PWD => {
                    len = 2;
                }
                Type::LET => {
                    len = 3;
                }
                Type::WHEN => {
                    len = 4;
                }
                Type::LBRACE => {
                    continue;
                }
                Type::RBRACE => {
                    break;
                }
                _ => {}
            }
            while len > 0 {
                arguments.push(self.lexer.next_token());
                len -= 1;
            }
            statements.push(Statement { token, arguments });
        }
        statements
    }
}

