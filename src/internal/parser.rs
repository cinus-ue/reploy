use std::collections::HashMap;

use internal::error::ReployError;
use internal::lexer::Lexer;
use internal::token::{Token, Type};
use internal::{Recipe, Statement};

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
                        return Err(ReployError::InvalidRecipe(format!(
                            "Line {}: Missing target after TARGET",
                            token.line_num
                        )));
                    }
                    let mut arguments: Vec<Token> = Vec::new();
                    arguments.push(next_token);
                    recipe.task.push(Statement::Simple { token, arguments });
                }
                Type::SET => {
                    let k = self.lexer.next_token();
                    let v = self.lexer.next_token();
                    if k.token_type == Type::EOF || v.token_type == Type::EOF {
                        return Err(ReployError::InvalidRecipe(format!(
                            "Line {}: Incomplete SET statement",
                            token.line_num
                        )));
                    }
                    recipe.variables.insert(k.literal, v.literal);
                }
                Type::TASK => {
                    recipe.task.append(&mut self.parse_statement()?);
                }
                Type::LABEL => {
                    let label = self.lexer.next_token();
                    if label.token_type == Type::EOF {
                        return Err(ReployError::InvalidRecipe(format!(
                            "Line {}: Missing label name after LABEL",
                            token.line_num
                        )));
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
            match token.token_type {
                Type::FOR => {
                    statements.push(self.parse_for_loop()?);
                }
                Type::FORIN => {
                    statements.push(self.parse_forin_loop()?);
                }
                Type::WHILE => {
                    statements.push(self.parse_while()?);
                }
                Type::SET => {
                    let k = self.lexer.next_token();
                    let v = self.lexer.next_token();
                    if k.token_type == Type::EOF || v.token_type == Type::EOF {
                        return Err(ReployError::InvalidRecipe(format!(
                            "Line {}: Incomplete SET statement",
                            token.line_num
                        )));
                    }
                    statements.push(Statement::Simple {
                        token,
                        arguments: vec![k, v],
                    });
                }
                Type::RUN | Type::PRINT | Type::CALL => {
                    let mut arguments: Vec<Token> = Vec::new();
                    let mut len = 1;
                    while len > 0 {
                        let arg = self.lexer.next_token();
                        if arg.token_type == Type::EOF {
                            return Err(ReployError::InvalidRecipe(format!(
                                "Line {}: Incomplete statement at token: {}",
                                token.line_num, token.literal
                            )));
                        }
                        arguments.push(arg);
                        len -= 1;
                    }
                    statements.push(Statement::Simple { token, arguments });
                }
                Type::SND | Type::RCV | Type::ASK | Type::PWD => {
                    let mut arguments: Vec<Token> = Vec::new();
                    let mut len = 2;
                    while len > 0 {
                        let arg = self.lexer.next_token();
                        if arg.token_type == Type::EOF {
                            return Err(ReployError::InvalidRecipe(format!(
                                "Line {}: Incomplete statement at token: {}",
                                token.line_num, token.literal
                            )));
                        }
                        arguments.push(arg);
                        len -= 1;
                    }
                    statements.push(Statement::Simple { token, arguments });
                }
                Type::LET | Type::WAIT => {
                    let mut arguments: Vec<Token> = Vec::new();
                    let mut len = 3;
                    while len > 0 {
                        let arg = self.lexer.next_token();
                        if arg.token_type == Type::EOF {
                            return Err(ReployError::InvalidRecipe(format!(
                                "Line {}: Incomplete statement at token: {}",
                                token.line_num, token.literal
                            )));
                        }
                        arguments.push(arg);
                        len -= 1;
                    }
                    statements.push(Statement::Simple { token, arguments });
                }
                Type::WHEN => {
                    let condition = self.lexer.next_token();
                    if condition.token_type == Type::EOF {
                        return Err(ReployError::InvalidRecipe(format!(
                            "Line {}: Missing condition after WHEN",
                            token.line_num
                        )));
                    }

                    let lbrace = self.lexer.next_token();
                    if lbrace.token_type != Type::LBRACE {
                        return Err(ReployError::InvalidRecipe(format!(
                            "Line {}: Expected '{{' after WHEN condition",
                            lbrace.line_num
                        )));
                    }

                    let body = self.parse_statement()?;

                    statements.push(Statement::When { condition, body });
                }
                Type::END => {
                    statements.push(Statement::Simple {
                        token,
                        arguments: Vec::new(),
                    });
                }
                Type::LBRACE => {
                    continue;
                }
                Type::RBRACE => {
                    break;
                }
                _ => {}
            }
        }
        Ok(statements)
    }

    fn parse_for_loop(&mut self) -> Result<Statement, ReployError> {
        // Read loop variable
        let variable = self.lexer.next_token();
        if variable.token_type == Type::EOF {
            return Err(ReployError::InvalidRecipe(format!(
                "Line {}: Missing loop variable after FOR",
                variable.line_num
            )));
        }

        // Read start value
        let start = self.lexer.next_token();
        if start.token_type == Type::EOF {
            return Err(ReployError::InvalidRecipe(format!(
                "Line {}: Missing start value in FOR loop",
                start.line_num
            )));
        }

        // Read end value
        let end = self.lexer.next_token();
        if end.token_type == Type::EOF {
            return Err(ReployError::InvalidRecipe(format!(
                "Line {}: Missing end value in FOR loop",
                end.line_num
            )));
        }

        // Check for optional step
        let next = self.lexer.peek_token();
        let step = if next.token_type != Type::LBRACE {
            Some(self.lexer.next_token())
        } else {
            None
        };

        // Parse loop body
        let lbrace = self.lexer.next_token();
        if lbrace.token_type != Type::LBRACE {
            return Err(ReployError::InvalidRecipe(format!(
                "Line {}: Expected '{{' after FOR loop parameters",
                lbrace.line_num
            )));
        }

        let body = self.parse_statement()?;

        Ok(Statement::Loop {
            variable,
            start,
            end,
            step,
            body,
        })
    }

    fn parse_forin_loop(&mut self) -> Result<Statement, ReployError> {
        // Read loop variable
        let variable = self.lexer.next_token();
        if variable.token_type == Type::EOF {
            return Err(ReployError::InvalidRecipe(format!(
                "Line {}: Missing loop variable after FORIN",
                variable.line_num
            )));
        }

        // Read IN keyword
        let in_keyword = self.lexer.next_token();
        if in_keyword.token_type != Type::IN {
            return Err(ReployError::InvalidRecipe(format!(
                "Line {}: Expected 'IN' after FORIN variable",
                in_keyword.line_num
            )));
        }

        // Read list expression
        let list = self.lexer.next_token();
        if list.token_type == Type::EOF {
            return Err(ReployError::InvalidRecipe(format!(
                "Line {}: Missing list expression in FORIN loop",
                list.line_num
            )));
        }

        // Parse loop body
        let lbrace = self.lexer.next_token();
        if lbrace.token_type != Type::LBRACE {
            return Err(ReployError::InvalidRecipe(format!(
                "Line {}: Expected '{{' after FORIN parameters",
                lbrace.line_num
            )));
        }

        let body = self.parse_statement()?;

        Ok(Statement::ListLoop {
            variable,
            list,
            body,
        })
    }

    fn parse_while(&mut self) -> Result<Statement, ReployError> {
        // Read condition
        let condition = self.lexer.next_token();
        if condition.token_type == Type::EOF {
            return Err(ReployError::InvalidRecipe(format!(
                "Line {}: Missing condition after WHILE",
                condition.line_num
            )));
        }

        // Parse loop body
        let lbrace = self.lexer.next_token();
        if lbrace.token_type != Type::LBRACE {
            return Err(ReployError::InvalidRecipe(format!(
                "Line {}: Expected '{{' after WHILE condition",
                lbrace.line_num
            )));
        }

        let body = self.parse_statement()?;

        Ok(Statement::While { condition, body })
    }
}
