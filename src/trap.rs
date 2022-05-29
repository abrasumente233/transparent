use core::arch::asm;
use riscv::register::{mtvec::TrapMode, sstatus::Sstatus, stvec};

use crate::println;

#[derive(Debug, Copy, Clone)]
struct Frame {
    gprs: [usize; 32],
    sstatus: Sstatus,
    sepc: usize,
}

type HandlerFn = extern "C" fn(&mut Frame) -> !;

macro_rules! save_registers {
    () => {
        asm!(
            "subi sp, sp, {frame_size}",

            "sd x0,  0*8(sp)",
            "sd x1,  1*8(sp)",
            "#sd x2,  2*8(sp)",
            "sd x3,  3*8(sp)",
            "sd x4,  4*8(sp)",
            "sd x5,  5*8(sp)",
            "sd x6,  6*8(sp)",
            "sd x7,  7*8(sp)",
            "sd x8,  8*8(sp)",
            "sd x9,  9*8(sp)",
            "sd x10, 10*8(sp)",
            "sd x11, 11*8(sp)",
            "sd x12, 12*8(sp)",
            "sd x13, 13*8(sp)",
            "sd x14, 14*8(sp)",
            "sd x15, 15*8(sp)",
            "sd x16, 16*8(sp)",
            "sd x17, 17*8(sp)",
            "sd x18, 18*8(sp)",
            "sd x19, 19*8(sp)",
            "sd x20, 20*8(sp)",
            "sd x21, 21*8(sp)",
            "sd x22, 22*8(sp)",
            "sd x23, 23*8(sp)",
            "sd x24, 24*8(sp)",
            "sd x25, 25*8(sp)",
            "sd x26, 26*8(sp)",
            "sd x27, 27*8(sp)",
            "sd x28, 28*8(sp)",
            "sd x29, 29*8(sp)",
            "sd x30, 30*8(sp)",
            "sd x31, 31*8(sp)",

            "csrr s0, sstatus",
            "csrr s1, sepc",

            "sd s0, 32*8(sp)",
            "sd s1, 33*8(sp)",
            frame_size = const core::mem::size_of::<Frame>(),
        );
    };
}

macro_rules! restore_registers {
    () => {
        asm!(
            "ld s0, 32*8(sp)",
            "ld s1, 33*8(sp)",

            "csrw sstatus, s0",
            "csrw sepc,    s1",

            "#ld x0,  0*8(sp)",
            "ld x1,  1*8(sp)",
            "#ld x2,  2*8(sp)",
            "ld x3,  3*8(sp)",
            "ld x4,  4*8(sp)",
            "ld x5,  5*8(sp)",
            "ld x6,  6*8(sp)",
            "ld x7,  7*8(sp)",
            "ld x8,  8*8(sp)",
            "ld x9,  9*8(sp)",
            "ld x10, 10*8(sp)",
            "ld x11, 11*8(sp)",
            "ld x12, 12*8(sp)",
            "ld x13, 13*8(sp)",
            "ld x14, 14*8(sp)",
            "ld x15, 15*8(sp)",
            "ld x16, 16*8(sp)",
            "ld x17, 17*8(sp)",
            "ld x18, 18*8(sp)",
            "ld x19, 19*8(sp)",
            "ld x20, 20*8(sp)",
            "ld x21, 21*8(sp)",
            "ld x22, 22*8(sp)",
            "ld x23, 23*8(sp)",
            "ld x24, 24*8(sp)",
            "ld x25, 25*8(sp)",
            "ld x26, 26*8(sp)",
            "ld x27, 27*8(sp)",
            "ld x28, 28*8(sp)",
            "ld x29, 29*8(sp)",
            "ld x30, 30*8(sp)",
            "ld x31, 31*8(sp)",

            "csrr x0, sstatus",
            "csrr x1, sepc",

            "addi sp, sp, {frame_size}",
            frame_size = const core::mem::size_of::<Frame>(),
            );
    };
}

macro_rules! handler {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                save_registers!();
                asm!(
                    "mv a0, sp",
                    "jalr {handler_fn}",
                    handler_fn = in(reg) $name as HandlerFn,);
                restore_registers!();
                asm!("sret", options(noreturn));
            };
        }
        wrapper
    }}
}

pub(crate) fn init() {
    unsafe {
        stvec::write(handler!(handle_trap) as usize, TrapMode::Direct);
    }
}

extern "C" fn handle_trap(frame: &mut Frame) -> ! {
    println!("{:?}", frame);
    loop {}
}
