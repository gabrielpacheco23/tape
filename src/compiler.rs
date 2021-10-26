use crate::parser::Parser;
use crate::scanner::{Scanner, TokenType};
use crate::vm::{BracketKind, OpCode, Program};

pub struct Compiler {
    pub parser: Parser,
    pub program: Vec<OpCode>,
    pub tape_name: String,
    pub idx_name: String,
}

impl Compiler {
    pub fn new(code: &str) -> Self {
        Self {
            parser: Parser::new(Scanner::new(code)),
            program: vec![],
            tape_name: "tape".to_owned(),
            idx_name: "idx".to_owned(),
        }
    }

    fn emit(&mut self, op: OpCode) {
        self.program.push(op);
    }

    fn make_tape_variable(&mut self) {
        self.parser.consume(TokenType::Ident);
        let var_name = self.parser.previous.clone().lexeme;

        if !self.parser.matches(TokenType::LeftBrace) {
            self.default_make_tape();
            self.parser.consume(TokenType::Colon);
            self.idx_name = var_name;
            self.parser.consume_fixed(TokenType::Ident, "idx");
            return;
        }

        self.parser.consume(TokenType::Number);
        let num_token = self.parser.previous.clone();
        let num = match num_token.lexeme.parse::<usize>() {
            Ok(num) => num,
            Err(_) => self.parser.error_at_current("Could not parse number"),
        };

        self.tape_name = var_name.to_owned();
        self.parser.consume(TokenType::RightBrace);
        self.emit(OpCode::MakeTape(num));
    }

    fn default_make_tape(&mut self) {
        self.emit(OpCode::MakeTape(30_000));
    }

    fn make_tape_decl(&mut self) {
        if self.parser.matches(TokenType::Make) {
            self.make_tape_variable();
        } else {
            self.default_make_tape();
        }
    }

    fn make_idx_variable(&mut self) {
        self.parser.consume(TokenType::Ident);
        let var_name = self.parser.previous.clone().lexeme;

        self.parser.consume(TokenType::Colon);
        self.idx_name = var_name.to_owned();
        self.parser.consume_fixed(TokenType::Ident, "idx");
    }

    fn make_idx_decl(&mut self) {
        if self.parser.matches(TokenType::Make) {
            self.make_idx_variable();
        }
    }

    fn incr_stmt(&mut self) {
        self.parser.consume(TokenType::Ident);
        let ident = self.parser.previous.clone();

        if self.parser.matches(TokenType::LeftBrace) {
            if self.tape_name != ident.lexeme {
                self.parser
                    .error_at_current(&format!("`{}` not defined", &ident.lexeme));
            }

            self.parser.consume(TokenType::Ident);
            let idx = self.parser.previous.clone();

            if self.idx_name != idx.lexeme {
                self.parser
                    .error_at_current(&format!("`{}` not defined", &idx.lexeme));
            }

            self.parser.consume(TokenType::RightBrace);
            self.emit(OpCode::IncrCell);
        } else {
            if self.idx_name != ident.lexeme {
                self.parser
                    .error_at_current(&format!("`{}` not defined", &ident.lexeme));
            }

            self.emit(OpCode::IncrPtr);
        }
    }

    fn decr_stmt(&mut self) {
        self.parser.consume(TokenType::Ident);
        let ident = self.parser.previous.clone();

        if self.parser.matches(TokenType::LeftBrace) {
            if self.tape_name != ident.lexeme {
                self.parser
                    .error_at_current(&format!("`{}` not defined", &ident.lexeme));
            }

            self.parser.consume(TokenType::Ident);
            let idx = self.parser.previous.clone();

            if self.idx_name != idx.lexeme {
                self.parser
                    .error_at_current(&format!("`{}` not defined", &idx.lexeme));
            }

            self.parser.consume(TokenType::RightBrace);
            self.emit(OpCode::DecrCell);
        } else {
            if self.idx_name != ident.lexeme {
                self.parser
                    .error_at_current(&format!("`{}` not defined", &ident.lexeme));
            }

            self.emit(OpCode::DecrPtr);
        }
    }

    fn putch_stmt(&mut self) {
        self.emit(OpCode::PrintChar);
    }

    fn getch_stmt(&mut self) {
        self.emit(OpCode::GetChar);
    }

    fn loop_block(&mut self) {
        while !self.parser.check(TokenType::RightParen) && !self.parser.check(TokenType::EOF) {
            self.statement();
        }

        self.parser.consume(TokenType::RightParen);
    }

    fn loop_stmt(&mut self) {
        self.parser.consume(TokenType::LeftParen);
        let loop_start = self.program.len();

        let exit_jump = self.emit_jump(OpCode::Jump(0, BracketKind::Close));
        self.loop_block();

        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        //let offset = self.program.len() - loop_start + 1;
        let offset = self.program.len() - loop_start;
        self.emit(OpCode::Jump(offset, BracketKind::Open));
    }

    fn emit_jump(&mut self, op: OpCode) -> usize {
        self.emit(op);
        self.program.len() - 1
    }

    fn patch_jump(&mut self, offset: usize) {
        //let jump = self.program.len() - offset - 1;
        let jump = self.program.len() - offset;

        let opcode = match self.program[offset] {
            OpCode::Jump(_, kind) => OpCode::Jump(jump, kind),
            _ => panic!("Jump was tried to be patched, opcode was not a jump!"),
        };

        self.program[offset] = opcode;
    }

    fn plus_stmt(&mut self) {
        self.parser.consume(TokenType::Number);
        let num_token = self.parser.previous.clone();
        let num = match num_token.lexeme.parse::<usize>() {
            Ok(num) => num,
            Err(_) => self.parser.error_at_current("Could not parse number"),
        };

        let last_op = self
            .program
            .last()
            .expect("Cannot repeat invalid statement")
            .clone();

        for _ in 0..num {
            self.emit(last_op);
        }
    }

    // nothing for a while
    fn debug_stmt(&mut self) {}

    fn statement(&mut self) {
        if self.parser.matches(TokenType::Incr) {
            self.incr_stmt();
        } else if self.parser.matches(TokenType::Decr) {
            self.decr_stmt();
        } else if self.parser.matches(TokenType::Putch) {
            self.putch_stmt();
        } else if self.parser.matches(TokenType::Getch) {
            self.getch_stmt();
        } else if self.parser.matches(TokenType::Loop) {
            self.loop_stmt();
        } else if self.parser.matches(TokenType::Debug) {
            self.debug_stmt();
        } else if self.parser.matches(TokenType::Plus) {
            self.plus_stmt();
        } else {
            self.parser.advance();
        }
    }

    pub fn compile(&mut self) -> Program {
        self.parser.advance();
        // always handles `make`s before everything
        self.make_tape_decl();
        self.make_idx_decl();

        while !self.parser.matches(TokenType::EOF) {
            self.statement();
        }

        self.program.clone()
    }
}
