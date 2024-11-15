use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};
use std::io::{BufRead, Read, Write};
use std::mem;
use std::slice;
use std::u8;

use crate::vm::{BracketKind, OpCode, Program};

const TAPE_SIZE: usize = 30_000;

macro_rules! my_dynasm {
    ($ops:ident $($t:tt)*) => {
        dynasm!($ops
            ; .arch x64
            ; .alias a_state, rcx
            ; .alias a_current, rdx
            ; .alias a_begin, r8
            ; .alias a_end, r9
            ; .alias retval, rax
            $($t)*
        )
    }
}

macro_rules! prologue {
    ($ops:ident) => {{
        let start = $ops.offset();
        my_dynasm!($ops
            ; sub rsp, 0x28
            ; mov [rsp + 0x30], rcx
            ; mov [rsp + 0x40], r8
            ; mov [rsp + 0x48], r9
        );
        start
    }};
}

macro_rules! epilogue {
    ($ops:ident, $e:expr) => {my_dynasm!($ops
        ; mov retval, $e
        ; add rsp, 0x28
        ; ret
    );};
}

macro_rules! call_extern {
    ($ops:ident, $addr:expr) => {my_dynasm!($ops
        ; mov [rsp + 0x38], rdx
        ; mov rax, QWORD $addr as _
        ; call rax
        ; mov rcx, [rsp + 0x30]
        ; mov rdx, [rsp + 0x38]
        ; mov r8,  [rsp + 0x40]
        ; mov r9,  [rsp + 0x48]
    );};
}

pub struct JitState<'a> {
    pub input: Box<dyn BufRead + 'a>,
    pub output: Box<dyn Write + 'a>,
    pub tape: [u8; TAPE_SIZE],
}

pub struct JitCode {
    code: dynasmrt::ExecutableBuffer,
    start: dynasmrt::AssemblyOffset,
}

pub fn jit_compile(program: &Program) -> Result<JitCode, &'static str> {
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let mut loops = vec![];
    let start = prologue!(ops);

    for (_i, op) in program.iter().enumerate() {
        match op {
            OpCode::MakeTape(_) => {}
            OpCode::IncrPtr => {
                let amount = 1; // TODO: make the amount accumulate to improve performance
                my_dynasm!(ops
                    ; add a_current, (amount % TAPE_SIZE) as _
                    ; cmp a_current, a_end
                    ; jb >wrap
                    ; sub a_current, TAPE_SIZE as _
                    ;wrap:
                );
            }
            OpCode::DecrPtr => {
                let amount = 1; // TODO: make the amount accumulate to improve performance
                my_dynasm!(ops
                    ; sub a_current, (amount % TAPE_SIZE) as _
                    ; cmp a_current, a_begin
                    ; jae >wrap
                    ; add a_current, TAPE_SIZE as _
                    ;wrap:
                );
            }
            OpCode::IncrCell => {
                let amount = 1; // TODO: make the amount accumulate to improve performance
                if amount > u8::MAX as usize {
                    return Err("An overflow occurred");
                }
                my_dynasm!(ops
                    ; add BYTE [a_current], amount as _
                    ; jo ->overflow
                );
            }
            OpCode::DecrCell => {
                let amount = 1; // TODO: make the amount accumulate to improve performance
                if amount > u8::MAX as usize {
                    return Err("An overflow occurred");
                }
                my_dynasm!(ops
                    ; sub BYTE [a_current], amount as _
                    ; jo ->overflow
                );
            }
            OpCode::PrintChar => {
                my_dynasm!(ops
                    ;; call_extern!(ops, JitState::putchar)
                    ; cmp al, 0
                    ; jnz ->io_failure
                );
            }
            OpCode::GetChar => {
                my_dynasm!(ops
                    ;; call_extern!(ops, JitState::getchar)
                    ; cmp al, 0
                    ; jnz ->io_failure
                );
            }
            OpCode::Debug => todo!(),

            #[allow(unused)]
            OpCode::Jump(ref offset, kind) => match kind {
                BracketKind::Close => {
                    // TODO: handle the [-] specific case

                    let backward_label = ops.new_dynamic_label();
                    let forward_label = ops.new_dynamic_label();
                    loops.push((backward_label, forward_label));
                    my_dynasm!(ops
                        ; cmp BYTE [a_current], 0
                        ; jz =>forward_label
                        ;=>backward_label
                    );
                }
                BracketKind::Open => {
                    if let Some((backward_label, forward_label)) = loops.pop() {
                        my_dynasm!(ops
                            ; cmp BYTE [a_current], 0
                            ; jnz =>backward_label
                            ;=>forward_label
                        );
                    } else {
                        return Err("loop without closing delimiter ')'");
                    }
                }
            },
        }
    }

    if loops.len() != 0 {
        return Err("[ without matching ]");
    }
    my_dynasm!(ops
        ;; epilogue!(ops, 0)
        ;->overflow:
        ;; epilogue!(ops, 1)
        ;->io_failure:
        ;; epilogue!(ops, 2)
    );

    let code = ops.finalize().unwrap();
    Ok(JitCode { code, start })
}

pub fn run_jit(state: &mut JitState, jit_code: JitCode) -> Result<(), &'static str> {
    let f: extern "win64" fn(*mut JitState, *mut u8, *mut u8, *const u8) -> u8 =
        unsafe { mem::transmute(jit_code.code.ptr(jit_code.start)) };
    let start = state.tape.as_mut_ptr();
    let end = unsafe { start.offset(TAPE_SIZE as isize) };
    let res = f(state, start, start, end);
    if res == 0 {
        Ok(())
    } else if res == 1 {
        Err("An overflow occurred")
    } else if res == 2 {
        Err("IO error")
    } else {
        panic!("Unknown error code");
    }
}

impl<'a> JitState<'a> {
    unsafe extern "win64" fn getchar(state: *mut JitState, cell: *mut u8) -> u8 {
        let state = &mut *state;
        let err = state.output.flush().is_err();
        (state
            .input
            .read_exact(slice::from_raw_parts_mut(cell, 1))
            .is_err()
            || err) as u8
    }

    unsafe extern "win64" fn putchar(state: *mut JitState, cell: *mut u8) -> u8 {
        let state = &mut *state;
        state
            .output
            .write_all(slice::from_raw_parts(cell, 1))
            .is_err() as u8
    }

    pub fn new(input: Box<dyn BufRead + 'a>, output: Box<dyn Write + 'a>) -> JitState<'a> {
        JitState {
            input,
            output,
            tape: [0; TAPE_SIZE],
        }
    }
}
