use crate::tape_struct::Tape;
use std::io::Read;

#[derive(Debug, Clone, Copy)]
pub enum BracketKind {
    Open,
    Close,
}

#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    #[allow(unused)]
    MakeTape(usize),
    IncrPtr,
    DecrPtr,
    IncrCell,
    DecrCell,
    PrintChar,
    GetChar,
    Debug,
    Jump(usize, BracketKind),
}

pub type Program = Vec<OpCode>;

#[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn run(&mut self, program: &Program) -> Result<(), &'static str> {
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
                    } else {
                        return Err("An overflow occurred");
                    }
                }
                DecrCell => {
                    if self.tape[self.index] > 0 {
                        self.tape[self.index] -= 1;
                    } else {
                        return Err("An overflow occurred");
                    }
                }
                PrintChar => print!("{}", self.tape[self.index] as char),
                GetChar => {
                    self.tape[self.index] = std::io::stdin()
                        .bytes()
                        .next()
                        .and_then(|res| res.ok())
                        .unwrap();
                    //    let mut buf = [0; 2];    // 2 because of the "\n" char
                    //    std::io::stdin().read(&mut buf).unwrap();
                    //    self.tape[self.index] = buf[0];
                }
                Debug => {
                    println!("{:#?}", self.tape);
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
        Ok(())
    }
}
