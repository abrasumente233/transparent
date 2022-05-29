use core::arch::asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt},
    sstatus::{self, Sstatus},
    stvec,
};

use crate::{plic, print, println, timer};

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct Frame {
    gprs: [usize; 32],
    sstatus: Sstatus,
    sepc: usize,
}

//type HandlerFn = extern "C" fn(&mut Frame);

macro_rules! handler {
    ($name: ident) => {{
        #[repr(align(4))]
        #[naked]
        extern "C" fn wrapper() {
            unsafe {
                asm!(
                    "addi sp, sp, -{frame_size}",

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
                    "mv a0, sp",
                    "call {handler_fn}",

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

                    "addi sp, sp, {frame_size}",

                    "sret",

                    frame_size = const core::mem::size_of::<Frame>(),
                    handler_fn = sym $name,
                    options(noreturn)
                );
            };
        }
        wrapper
    }}
}

pub(crate) fn init() {
    unsafe {
        stvec::write(handler!(handle_trap) as usize, TrapMode::Direct);
        sstatus::set_sie();
    }
}

pub(crate) extern "C" fn handle_trap(frame: &mut Frame) {
    let scause = scause::read();
    match scause.cause() {
        scause::Trap::Interrupt(intr) => handle_interrupts(frame, intr),
        scause::Trap::Exception(except) => handle_exceptions(frame, except),
    }
}

pub(crate) fn handle_interrupts(frame: &mut Frame, intr: Interrupt) {
    match intr {
        Interrupt::UserSoft => todo!(),
        Interrupt::SupervisorSoft => todo!(),
        Interrupt::UserTimer => todo!(),
        Interrupt::SupervisorTimer => {
            print!(".");
            timer::set_next_timer()
        }
        Interrupt::UserExternal => todo!(),
        Interrupt::SupervisorExternal => plic::handle_interrupts(frame),
        Interrupt::Unknown => {
            println!("Trap frame: {:?}", frame);
            panic!("Unknown interrupt: {:?}", intr);
        }
    }
}

pub(crate) fn handle_exceptions(frame: &mut Frame, except: Exception) {
    match except {
        Exception::InstructionMisaligned => todo!(),
        Exception::InstructionFault => todo!(),
        Exception::IllegalInstruction => todo!(),
        Exception::Breakpoint => {
            println!("Breakpoint at 0x{:x}", frame.sepc);
            frame.sepc += 4;
        }
        //Exception::LoadFault => todo!(),
        Exception::StoreMisaligned => todo!(),
        //Exception::StoreFault => todo!(),
        Exception::UserEnvCall => todo!(),
        Exception::InstructionPageFault => todo!(),
        Exception::LoadPageFault => todo!(),
        Exception::StorePageFault => todo!(),
        _ | Exception::Unknown => {
            println!("Trap frame: {:?}", frame);
            panic!("Unknown exception: {:?}", except);
        }
    }
}

pub(crate) fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let saved_sie = sstatus::read().sie();

    // If the interrupts are enabled, disable them for now
    if saved_sie {
        unsafe {
            sstatus::clear_sie();
        }
    }

    let ret = f();

    if saved_sie {
        unsafe {
            sstatus::set_sie();
        }
    }

    ret
}
