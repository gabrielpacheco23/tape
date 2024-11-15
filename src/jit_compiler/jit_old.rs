use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};
use libc::{c_void, mmap, MAP_ANONYMOUS, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};
use std::io::{BufRead, Read, Write};
use std::mem;
use std::ptr;
use std::slice;
use std::u8;

#[allow(dead_code)]
pub fn jit_compile(&mut self, program: &Program) {
    let mut code: Vec<u8> = vec![];
    let mut backpatches: Vec<JitBackpatch> = vec![];
    let mut addrs: Vec<usize> = vec![];

    for (_i, op) in program.iter().enumerate() {
        addrs.push(code.len());

        match op {
            OpCode::MakeTape(_) => {}
            OpCode::IncrPtr => {
                // add rdi, 1 => 4883 c701
                code.push(0x48);
                code.push(0x83);
                code.push(0xc7);
                code.push(0x01);
            }
            OpCode::DecrPtr => {
                // sub rdi, 1 => 4883 ef01
                code.push(0x48);
                code.push(0x83);
                code.push(0xef);
                code.push(0x01);
            }
            OpCode::IncrCell => {
                // inc byte [rdi]
                code.push(0xfe);
                code.push(0x07);
            }
            OpCode::DecrCell => {
                // dec byte [rdi]
                code.push(0xfe);
                code.push(0x0f);
            }
            OpCode::PrintChar => {
                // push rdi
                code.push(0x57);
                // mov rax, 1 => 48c7 c001 0000 00
                code.extend_from_slice(&[0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00]);
                // mov rsi, rdi => 4889 fe
                code.extend_from_slice(&[0x48, 0x89, 0xfe]);
                // mov rdi, 1 => 48c7 c701 0000 00
                code.extend_from_slice(&[0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00]);
                // mov rdx, 1 => 48c7 c201 0000 00
                code.extend_from_slice(&[0x48, 0xc7, 0xc2, 0x01, 0x00, 0x00, 0x00]);
                // syscall => 0f05
                code.push(0x0f);
                code.push(0x05);
                // pop rdi => 5f
                code.push(0x5f);
            }
            OpCode::GetChar => todo!(),
            OpCode::Debug => todo!(),
            OpCode::Jump(ref offset, kind) => match kind {
                BracketKind::Open => {
                    // xor rax, rax
                    code.extend_from_slice(&[0x48, 0x31, 0xc0]);
                    // mov al, byte [rdi]
                    code.extend_from_slice(&[0x8a, 0x07]);
                    // test rax, rax
                    code.extend_from_slice(&[0x48, 0x85, 0xc0]);
                    // jz => 0f84 0000 0000
                    code.extend_from_slice(&[0x0f, 0x84]);
                    let op_addr = code.len();
                    code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
                    let src_byte_addr = code.len();

                    let bp = JitBackpatch {
                        operand_byte_addr: op_addr,
                        src_byte_addr,
                        dst_op_index: *offset,
                    };

                    backpatches.push(bp);
                }
                BracketKind::Close => {
                    // xor rax, rax
                    code.extend_from_slice(&[0x48, 0x31, 0xc0]);
                    // mov al, byte [rdi]
                    code.extend_from_slice(&[0x8a, 0x07]);
                    // test rax, rax
                    code.extend_from_slice(&[0x48, 0x85, 0xc0]);
                    // jnz => 0f85 0000 0000
                    code.extend_from_slice(&[0x0f, 0x85]);
                    let op_addr = code.len();
                    code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
                    let src_byte_addr = code.len();

                    let bp = JitBackpatch {
                        operand_byte_addr: op_addr,
                        src_byte_addr,
                        dst_op_index: *offset,
                    };

                    backpatches.push(bp);
                }
            },
        }
    }

    addrs.push(code.len());

    for (_i, bp) in backpatches.iter().enumerate() {
        // println!(
        //     "operand_byte_addr = {}, src_byte_addr = {}, dst_op_inex = {}, dst_byte_addr = {}",
        //     bp.operand_byte_addr, bp.src_byte_addr, bp.dst_op_index, addrs[bp.dst_op_index]
        // );

        let src_addr = bp.src_byte_addr as i32;
        let dst_addr = addrs[bp.dst_op_index] as i32;
        let operand = dst_addr - src_addr;

        // memcpy
        unsafe {
            std::ptr::copy_nonoverlapping(
                &code[bp.operand_byte_addr],
                (&operand as *const i32) as *mut u8,
                std::mem::size_of_val(&operand),
            )
        }
    }

    // ret
    code.push(0xc3);

    // println!("Addrs: {:?}", addrs);
    // println!("Code: {:02x?}", code);

    if let Err(why) = mmap_execode(&code) {
        eprintln!("ERROR: {why}");
        std::process::exit(1);
    }
}

struct JitBackpatch {
    operand_byte_addr: usize,
    src_byte_addr: usize,
    dst_op_index: usize,
}

fn mmap_execode(buffer: &[u8]) -> Result<(), std::io::Error> {
    let ptr = unsafe {
        mmap(
            ptr::null_mut(),
            buffer.len(),
            PROT_EXEC | PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        )
    };

    if ptr == libc::MAP_FAILED {
        return Err(std::io::Error::last_os_error());
    }

    // memcpy
    unsafe { ptr::copy_nonoverlapping(buffer.as_ptr(), ptr as *mut u8, buffer.len()) };
    let code: extern "win64" fn() -> c_void = unsafe { mem::transmute(ptr) };
    code();

    Ok(())
}
