mod compiler;
mod jit_compiler;
mod parser;
mod scanner;
mod tape_struct;
mod vm;

use clap::Parser;
use compiler::Compiler;
use jit_compiler::{jit_compile, run_jit, JitState};
use std::fs;
use std::io::stdin;
use std::io::stdout;
use std::io::BufReader;
use std::io::BufWriter;
use vm::{Program, Vm};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        long,
        short,
        action,
        default_value = "false",
        default_missing_value = "true"
    )]
    jit: bool,
    #[arg(
        long,
        short,
        action,
        default_value = "false",
        default_missing_value = "true"
    )]
    verbose: bool,
    file_path: std::path::PathBuf,
}

// TODO: improve the JIT compiler

fn main() -> Result<(), &'static str> {
    let args = Args::parse();

    let source_code = fs::read_to_string(args.file_path).expect("failed reading file");

    let mut compiler = Compiler::new(&source_code);
    let program = compiler.compile();
    // debug_show(&program);

    if args.jit {
        if args.verbose {
            println!("[Using JIT compiler]\n");
        }
        jit(&program)
    } else {
        if args.verbose {
            println!("[Using bytecode VM]\n");
        }
        let mut vm = Vm::new();
        vm.run(&program)
    }
}

fn jit(p: &Program) -> Result<(), &'static str> {
    let mut state = JitState::new(
        Box::new(BufReader::new(stdin())),
        Box::new(BufWriter::new(stdout())),
    );

    let jit_code = jit_compile(&p)?;
    run_jit(&mut state, jit_code)
}

#[allow(dead_code)]
fn debug_show(p: &Program) {
    println!("{:#?}", p);
}
