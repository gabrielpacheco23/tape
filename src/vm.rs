use std::io::Read;

use crate::tape_struct::Tape;

#[derive(Debug, Clone, Copy)]
pub enum BracketKind {
    Open,
    Close,
}

#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    MakeTape(usize),
    IncrPtr,
    DecrPtr,
    IncrCell,
    DecrCell,
    PrintChar,
    GetChar,
    Jump(usize, BracketKind),
}

pub type Program = Vec<OpCode>;

pub struct Vm {
    tape: Tape,
    index: usize,
}

impl Vm {
    pub fn new() -> Self {
        Vm {
            tape: Tape::new(),
            index: 0,
        }
    }

    pub fn run(&mut self, program: &Program) {
        use OpCode::*;

        let mut iter = 0;
        while iter < program.len() {
            match program[iter] {
                MakeTape(size) => {
                    self.tape.init(size);
                }
                IncrPtr => {
                    if self.index >= self.tape.size() {
                        println!(
                            "self.index out of range: `{}` is greater than `{}`",
                            self.index,
                            self.tape.size()
                        );
                        std::process::exit(1);
                    }
                    self.index += 1;
                }
                DecrPtr => {
                    if self.index > 0 {
                        self.index -= 1;
                    }
                }
                IncrCell => {
                    if self.tape[self.index] < u8::MAX {
                        self.tape[self.index] += 1;
                    }
                }
                DecrCell => {
                    if self.tape[self.index] > 0 {
                        self.tape[self.index] -= 1;
                    }
                }
                PrintChar => print!("{}", self.tape[self.index] as char),
                GetChar => {
                    self.tape[self.index] = std::io::stdin().bytes().next().unwrap().unwrap()
                }
                Jump(ref offset, kind) => match kind {
                    BracketKind::Open => {
                        if self.tape[self.index] != 0 {
                            iter -= *offset;
                            continue;
                        }
                    }
                    BracketKind::Close => {
                        if self.tape[self.index] == 0 {
                            iter += *offset;
                            continue;
                        }
                    }
                },
            }
            iter += 1;
        }
    }
}
