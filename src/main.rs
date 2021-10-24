// TODO: Create a new cell-based, brainfuck-like, esoteric language.

mod compiler;
mod parser;
mod scanner;
mod vm;

use compiler::Compiler;
use std::env;
use std::fs;
use vm::{Program, Vm};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Wrong number of arguments: got {}, expected 2.", args.len());
    }

    let source_code = fs::read_to_string(&args[1]).expect("failed reading file");

    let mut compiler = Compiler::new(&source_code);
    let program = compiler.compile();

    // debug_show(&program);

    let mut vm = Vm::new();
    vm.run(&program);
}

#[allow(dead_code)]
fn debug_show(p: &Program) {
    println!("{:#?}", p);
}
