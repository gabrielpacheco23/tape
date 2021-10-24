#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // keywords
    Make,
    Incr,
    Decr,
    Getch,
    Putch,
    Loop,
    Debug,
    // symbols
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Colon,
    Plus,
    // other
    Number,
    Ident,
    Error,
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub start: usize,
    pub len: usize,
    pub line: u32,
}

impl Token {
    pub fn new(scanner: &Scanner, typ: TokenType) -> Self {
        Self {
            typ,
            start: scanner.start,
            len: scanner.current - scanner.start,
            line: scanner.line,
            lexeme: scanner.source[scanner.start..scanner.current]
                .iter()
                .collect(),
        }
    }

    pub fn default() -> Self {
        Token {
            start: 0,
            line: 0,
            len: 0,
            typ: TokenType::Ident,
            lexeme: "<default>".to_owned(),
        }
    }
    pub fn error(start: usize, line: u32, msg: &str) -> Token {
        Token {
            start,
            line,
            len: msg.len(),
            typ: TokenType::Error,
            lexeme: msg.to_owned(),
        }
    }
}

pub struct Scanner {
    pub source: Vec<char>,
    pub start: usize,
    pub current: usize,
    pub line: u32,
    pub len: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
            len: source.len(),
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        while !self.is_at_end() {
            // self.start = self.current;
            tokens.push(self.scan_token());
        }

        tokens
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespaces();

        self.start = self.current;

        if self.is_at_end() {
            return Token::new(&self, TokenType::EOF);
        }

        let c = self.advance();
        if self.is_alpha(c) {
            return self.identifier();
        }

        if self.is_digit(c) {
            return self.number();
        }

        match c {
            '+' => Token::new(&self, TokenType::Plus),
            ':' => Token::new(&self, TokenType::Colon),
            '[' => Token::new(&self, TokenType::LeftBrace),
            ']' => Token::new(&self, TokenType::RightBrace),
            '(' => Token::new(&self, TokenType::LeftParen),
            ')' => Token::new(&self, TokenType::RightParen),
            _ => Token::error(self.start, self.line, "Unexpected character."),
        }
    }

    fn skip_whitespaces(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn number(&mut self) -> Token {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        Token::new(&self, TokenType::Number)
    }

    fn identifier(&mut self) -> Token {
        while self.is_alpha(self.peek()) || self.is_digit(self.peek()) {
            self.advance();
        }
        Token::new(&self, self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        match self.source[self.start] {
            'm' => self.check_keyword(1, "ake", TokenType::Make),
            'i' => self.check_keyword(1, "ncr", TokenType::Incr),
            'g' => self.check_keyword(1, "etch", TokenType::Getch),
            'p' => self.check_keyword(1, "utch", TokenType::Putch),
            'l' => self.check_keyword(1, "oop", TokenType::Loop),
            'd' => {
                if self.current - self.start > 1 {
                    match self.source[self.start + 1] {
                        'e' => match self.source[self.start + 2] {
                            'b' => self.check_keyword(3, "ug", TokenType::Debug),
                            'c' => self.check_keyword(3, "r", TokenType::Decr),
                            _ => TokenType::Ident,
                        },
                        _ => TokenType::Ident,
                    }
                } else {
                    TokenType::Ident
                }
            }

            _ => TokenType::Ident,
        }
    }

    fn check_keyword(&self, start: usize, rest: &str, ty: TokenType) -> TokenType {
        let length = rest.len();
        let start_index = self.start + start;
        let end_index = start_index + length;
        let substr: String = self.source[start_index..end_index].into_iter().collect();

        if (self.current - self.start == start + length) && (substr == rest) {
            return ty;
        }
        TokenType::Ident
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current]
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current + 1]
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }

    //    pub fn match_token(&mut self, expected: char) -> bool {
    //        if self.is_at_end() || self.source[self.current] != expected {
    //            return false;
    //        }
    //
    //        self.current += 1;
    //        true
    //    }

    fn is_alpha(&self, c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.len || self.source[self.current] == '\0'
    }
}
