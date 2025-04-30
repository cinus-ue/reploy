use std::collections::HashMap;

use internal::{Recipe, Statement};
use internal::error::ReployError;
use internal::lexer::Lexer;
use internal::token::{Token, Type};

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        Parser { lexer }
    }

    pub fn parse(&mut self) -> Result<Recipe, ReployError> {
        let mut recipe = Recipe {
            task: Vec::new(),
            variables: HashMap::new(),
            labels: HashMap::new(),
        };
        loop {
            let token = self.lexer.next_token();
            match token.token_type {
                Type::TARGET => {
                    let next_token = self.lexer.next_token();
                    if next_token.token_type == Type::EOF {
                    return Err(ReployError::InvalidRecipe(
                        format!("Line {}: Missing target after TARGET", token.line_num)
                    ));
                    }
                    let mut arguments: Vec<Token> = Vec::new();
                    arguments.push(next_token);
                    recipe.task.push(Statement { token, arguments });
                }
                Type::SET => {
                    let k = self.lexer.next_token();
                    let v = self.lexer.next_token();
                    if k.token_type == Type::EOF || v.token_type == Type::EOF {
                    return Err(ReployError::InvalidRecipe(
                        format!("Line {}: Incomplete SET statement", token.line_num)
                    ));
                    }
                    recipe.variables.insert(k.literal, v.literal);
                }
                Type::TASK => {
                    recipe.task.append(&mut self.parse_statement()?);
                }
                Type::LABEL => {
                    let label = self.lexer.next_token();
                    if label.token_type == Type::EOF {
                    return Err(ReployError::InvalidRecipe(
                        format!("Line {}: Missing label name after LABEL", token.line_num)
                    ));
                    }
                    recipe.labels.insert(label.literal, self.parse_statement()?);
                }
                Type::EOF => {
                    break;
                }
                _ => {}
            }
        }
        Ok(recipe)
    }

    fn parse_statement(&mut self) -> Result<Vec<Statement>, ReployError> {
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
                Type::LET | Type::WAIT => {
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
                let arg = self.lexer.next_token();
                if arg.token_type == Type::EOF {
                    return Err(ReployError::InvalidRecipe(
                        format!("Line {}: Incomplete statement at token: {}", token.line_num, token.literal)
                    ));
                }
                arguments.push(arg);
                len -= 1;
            }
            statements.push(Statement { token, arguments });
        }
        Ok(statements)
    }
}

