use std::str::Chars;

use internal::token::{lookup_identifier, Token, Type};

const EOF_CHAR: char = '\0';
//const CR: char = '\u{000D}';
const LF: char = '\u{000A}';

pub struct Lexer {
    initial_len: usize,
    read_len: usize,
    line_num: usize,
    char: char,
    source_code: String,
}

impl Lexer {
    pub fn new(mut input: String) -> Lexer {
        if has_crlf_line_endings(&input) {
            input = input.replace("\r\n", "\n");
        }
        Lexer {
            initial_len: input.len(),
            read_len: 0,
            line_num: 0,
            char: input.chars().nth(0).unwrap_or(EOF_CHAR),
            source_code: input,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.char == '#' {
            self.skip_comment();
            return self.next_token();
        }
        return match self.char {
            EOF_CHAR => Token {
                literal: String::new(),
                line_num: self.line_num,
                token_type: Type::EOF,
            },
            '"' => Token {
                literal: self.read_string(),
                line_num: self.line_num,
                token_type: Type::STRING,
            },
            '{' => {
                if self.peek_char() == '{' {
                    self.read_char(); // Skip second '{'
                    Token {
                        literal: self.read_template_expression(),
                        line_num: self.line_num,
                        token_type: Type::EXPRESSION,
                    }
                } else {
                    let identifier = self.read_identifier();
                    Token {
                        literal: identifier.clone(),
                        line_num: self.line_num,
                        token_type: lookup_identifier(identifier),
                    }
                }
            }
            _ => {
                let identifier = self.read_identifier();
                Token {
                    literal: identifier.clone(),
                    line_num: self.line_num,
                    token_type: lookup_identifier(identifier),
                }
            }
        };
    }

    fn read_template_expression(&mut self) -> String {
        let mut chars: Vec<char> = vec!['{', '{'];

        while !self.is_eof() {
            self.read_char();
            if self.char == '}' && self.peek_char() == '}' {
                chars.push(self.char);
                self.read_char();
                chars.push(self.char);
                break;
            } else {
                chars.push(self.char);
            }
        }
        // Skip the last '}'
        if !is_whitespace(self.char) {
            self.read_char();
        }

        return chars.iter().collect::<String>();
    }

    fn is_eof(&self) -> bool {
        self.read_len == self.initial_len
    }

    fn chars(&self) -> Chars {
        self.source_code.chars().clone()
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
            // unterminated string
            if self.char == EOF_CHAR {
                break;
            }
            if self.char == '\\' {
                if is_lf(self.peek_char()) {
                    self.skip_source_code(1);
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

    fn skip_source_code(&mut self, mut len: usize) {
        while len > 0 {
            self.read_char();
            len -= 1;
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
        // new line
        if is_lf(self.char) {
            self.line_num += 1;
        }
    }

    pub fn peek_token(&mut self) -> Token {
        let saved_len = self.read_len;
        let saved_char = self.char;
        let saved_line_num = self.line_num;

        let token = self.next_token();

        self.read_len = saved_len;
        self.char = saved_char;
        self.line_num = saved_line_num;

        token
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

fn has_crlf_line_endings(s: &str) -> bool {
    // Only check the first line.
    if let Some(lf) = s.find('\n') {
        s[..lf].ends_with('\r')
    } else {
        false
    }
}

// unix style
fn is_lf(c: char) -> bool {
    return c == LF;
}

// windows style
//fn is_crlf(c1: char, c2: char) -> bool {
//    return c1 == CR && c2 == LF;
//}

fn is_end(c: char) -> bool {
    return c == EOF_CHAR;
}
