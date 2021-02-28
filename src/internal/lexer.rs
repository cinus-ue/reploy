use std::str::Chars;

use internal::token::{lookup_identifier, Token, Type};

const EOF_CHAR: char = '\0';
const CR: char = '\u{000D}';
const LF: char = '\u{000A}';


pub struct Lexer<'a> {
    initial_len: usize,
    read_len: usize,
    char: char,
    chars: Chars<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &str) -> Lexer {
        Lexer {
            initial_len: input.len(),
            read_len: 0,
            char: input.chars().nth(0).unwrap(),
            chars: input.chars(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.char == '#' {
            self.skip_comment();
            return self.next_token();
        }
        return match self.char {
            EOF_CHAR => {
                Token {
                    literal: String::from("EOF"),
                    token_type: Type::EOF,
                }
            }
            '"' => {
                Token {
                    literal: self.read_string(),
                    token_type: Type::STRING,
                }
            }
            _ => {
                let identifier = self.read_identifier();
                Token {
                    literal: identifier.clone(),
                    token_type: lookup_identifier(identifier),
                }
            }
        };
    }

    fn is_eof(&self) -> bool {
        self.read_len == self.initial_len
    }

    fn chars(&self) -> Chars<'a> {
        self.chars.clone()
    }

    fn nth_char(&self, n: usize) -> char {
        self.chars().nth(n).unwrap_or(EOF_CHAR)
    }

    fn read_string(&mut self) -> String {
        let mut chars: Vec<char> = Vec::new();
        loop {
            self.read_char();
            if self.char == '"' {
                self.read_char();
                break;
            }
            if self.char == EOF_CHAR {
                // unterminated string
                break;
            }
            if self.char == '\\' {
                if self.peek_char() == LF {
                    self.read_char();
                    continue;
                }
                if self.peek_char() == CR && self.nth_char(self.read_len + 2) == LF {
                    self.read_char();
                    self.read_char();
                    continue;
                }

                self.read_char();

                if self.char == 'n' {
                    self.char = '\n'
                }
                if self.char == 'r' {
                    self.char = '\r'
                }
                if self.char == 't' {
                    self.char = '\t'
                }
                if self.char == '"' {
                    self.char = '"'
                }
                if self.char == '\\' {
                    self.char = '\\'
                }
            }
            chars.push(self.char);
        }
        return chars.iter().collect::<String>();
    }

    fn read_identifier(&mut self) -> String {
        let mut chars: Vec<char> = Vec::new();
        while is_identifier(self.char) {
            chars.push(self.char);
            self.read_char();
        }
        return chars.iter().collect::<String>();
    }

    fn skip_comment(&mut self) {
        while self.char != LF && !self.is_eof() {
            self.read_char();
        }
        self.skip_whitespace();
    }

    fn skip_whitespace(&mut self) {
        while is_whitespace(self.char) && !self.is_eof() {
            self.read_char();
        }
    }

    fn peek_char(&self) -> char {
        return self.nth_char(self.read_len + 1);
    }


    fn read_char(&mut self) {
        if self.read_len < self.initial_len {
            self.read_len += 1;
        }
        self.char = self.nth_char(self.read_len);
    }
}


fn is_whitespace(c: char) -> bool {
    // This is Pattern_White_Space.
    //
    // Note that this set is stable (ie, it doesn't change with different
    // Unicode versions), so it's ok to just hard-code the values.

    match c {
        // Usual ASCII suspects
        | '\u{0009}' // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
        => true,
        _ => false,
    }
}

fn is_identifier(c: char) -> bool {
    return !is_whitespace(c) && !is_end(c);
}

fn is_end(c: char) -> bool {
    return c == EOF_CHAR;
}

