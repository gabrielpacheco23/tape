use crate::scanner::{Scanner, Token, TokenType};

pub struct Parser {
    pub current: Token,
    pub previous: Token,
    pub scanner: Scanner,
}

impl Parser {
    pub fn new(scanner: Scanner) -> Self {
        Self {
            scanner,
            previous: Token::default(),
            current: Token::default(),
        }
    }

    pub fn advance(&mut self) {
        self.previous = self.current.clone();

        loop {
            self.current = self.scanner.scan_token();
            if self.current.typ != TokenType::Error {
                break;
            }

            self.error_at_current(&self.current.lexeme.clone());
        }
    }

    pub fn consume(&mut self, typ: TokenType) {
        if self.current.typ == typ {
            self.advance();
        } else {
            let msg = &format!("Expected `[`, found: {}", self.current.lexeme);
            self.error_at_current(msg);
        }
    }

    pub fn consume_fixed(&mut self, typ: TokenType, val: &str) {
        if self.current.typ == typ && &self.current.lexeme == val {
            self.advance();
        } else {
            let msg = &format!("Expected `[`, found: {}", self.current.lexeme);
            self.error_at_current(msg);
        }
    }

    pub fn check(&self, typ: TokenType) -> bool {
        self.current.typ == typ
    }

    pub fn matches(&mut self, typ: TokenType) -> bool {
        if !self.check(typ) {
            return false;
        }
        self.advance();
        true
    }

    pub fn error_at_current(&self, err: &str) -> ! {
        println!("ERROR: {} at line {}", err, self.current.line);
        std::process::exit(1);
    }
}
