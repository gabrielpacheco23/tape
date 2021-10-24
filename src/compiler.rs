#![allow(unused)]

use crate::parser::Parser;
use crate::scanner::{Scanner, Token, TokenType};
use crate::vm::{BracketKind, OpCode, Program};
use std::{collections::HashMap, ops::Index};

pub struct Compiler {
    pub parser: Parser,
    pub program: Vec<OpCode>,
    pub tape_name: String,
    pub idx_name: String,
    brace_map: BraceMap,
}

impl Compiler {
    pub fn new(code: &str) -> Self {
        Self {
            parser: Parser::new(Scanner::new(code)),
            program: vec![],
            tape_name: "tape".to_owned(),
            idx_name: "idx".to_owned(),
            brace_map: BraceMap::new(),
        }
    }

    // ==================
    // +++++ [
    //   > ++++++++++++
    //   > ++
    //   << -
    // ]
    //
    // > +++++ .
    // > .
    //
    // TODO: cell-based language
    // ===================
    //  make tape[3]
    //  make ptr: idx
    //
    //  incr tape[ptr]
    //  +4
    //
    //  loop (
    //      incr ptr
    //      incr tape[ptr]
    //      +11
    //
    //      incr ptr
    //      incr tape[ptr]
    //      +1
    //
    //      decr ptr
    //      +1
    //      decr tape[ptr]
    //  )
    //
    //  incr ptr
    //  incr tape[ptr]
    //  +4
    //  putch
    //
    //  incr ptr
    //  putch
    // =====================
    //

    fn emit(&mut self, op: OpCode) {
        self.program.push(op);
    }

    fn declare_vars(&mut self) {
        self.parser.consume(TokenType::Ident);
        let var_name = self.parser.previous.clone().lexeme;

        if self.parser.matches(TokenType::LeftBrace) {
            self.parser.consume(TokenType::Number);
            let num_token = self.parser.previous.clone();
            let num = match num_token.lexeme.parse::<usize>() {
                Ok(num) => num,
                Err(err) => {
                    self.parser.error_at_current("Could not parse number");
                    return;
                }
            };

            self.tape_name = var_name;
            self.parser.consume(TokenType::RightBrace);
            self.emit(OpCode::MakeTape(num));
        } else if self.parser.matches(TokenType::Colon) {
            self.idx_name = var_name;
            self.parser.consume_fixed(TokenType::Ident, "idx");
        }
    }

    fn default_make(&mut self) {
        self.emit(OpCode::MakeTape(30_000));
    }

    // emit `make tape[n]` and `make ptr: idx`
    // TODO: make this work, going wrong
    fn make_decls(&mut self) {
        if self.parser.matches(TokenType::Make) {
            self.declare_vars();
        } else {
            self.default_make();
            self.statement();
        }
    }

    fn incr_stmt(&mut self) {
        self.parser.consume(TokenType::Ident);
        let ident = self.parser.previous.clone();

        if self.parser.matches(TokenType::LeftBrace) {
            if self.tape_name != ident.lexeme {
                self.parser
                    .error_at_current(&format!("`{}` not defined", &ident.lexeme));
                return;
            }
            self.parser.consume(TokenType::Ident);
            let idx = self.parser.previous.clone();

            if self.idx_name != idx.lexeme {
                self.parser
                    .error_at_current(&format!("`{}` not defined", &idx.lexeme));
                return;
            }

            self.parser.consume(TokenType::RightBrace);
            self.emit(OpCode::IncrCell);
        } else {
            if self.idx_name != ident.lexeme {
                self.parser
                    .error_at_current(&format!("`{}` not defined", &ident.lexeme));
                return;
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
                return;
            }
            self.parser.consume(TokenType::Ident);
            let idx = self.parser.previous.clone();

            if self.idx_name != idx.lexeme {
                self.parser
                    .error_at_current(&format!("`{}` not defined", &idx.lexeme));
                return;
            }

            self.parser.consume(TokenType::RightBrace);
            self.emit(OpCode::DecrCell);
        } else {
            if self.idx_name != ident.lexeme {
                self.parser
                    .error_at_current(&format!("`{}` not defined", &ident.lexeme));
                return;
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
        let mut loop_start = self.program.len();

        let exit_jump = self.emit_jump(OpCode::Jump(0, BracketKind::Close));
        self.loop_block();

        //self.emit(OpCode::Jump(offset, BracketKind::Open));
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
            Err(err) => {
                self.parser.error_at_current("Could not parse number");
                return;
            }
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
            // panic!("Unknow statement: {:#?}", &self.parser.current);
            self.parser.advance();
        }
    }

    pub fn compile(&mut self) -> Program {
        // verify if we have a make tape decl
        self.make_decls();

        //self.parser.advance();
        while !self.parser.matches(TokenType::EOF) {
            self.statement();
        }

        self.program.clone()
    }
}

#[derive(Debug)]
struct BraceMap {
    map: HashMap<usize, usize>,
}

impl BraceMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn set(&mut self, key: usize, value: usize) {
        self.map.insert(key, value);
    }

    fn get(&self, key: usize) -> &usize {
        self.map.get(&key).unwrap()
    }

    fn build(code: &[Token]) -> Self {
        let mut stack = Vec::new();
        let mut brace_map = BraceMap::new();

        for i in 0..code.len() {
            if code[i].typ == TokenType::LeftParen {
                stack.push(i);
            }

            if code[i].typ == TokenType::RightParen {
                if let Some(start) = stack.pop() {
                    brace_map.set(start, i);
                    brace_map.set(i, start);
                }
            }
        }

        brace_map
    }
}

impl Index<usize> for BraceMap {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}
