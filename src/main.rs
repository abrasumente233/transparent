#![no_std]
#![no_main]
#![feature(naked_functions, core_intrinsics, asm_const, asm_sym, fn_align)]

use core::arch::asm;

use riscv::asm::wfi;
use sbi::hart_state_management::hart_status;

mod assembly;
mod console;
mod panic;
mod plic;
mod timer;
mod trap;
mod uart;

#[no_mangle]
pub fn main() -> ! {
    trap::init();
    uart::init();
    plic::init();
    timer::init();

    println!("Hello, world!");
    println!("hart #0 status: {:?}", hart_status(0));

    unsafe {
        asm!("addi a0, a0, 0");
        asm!("ebreak");
        asm!("addi a0, a0, 0");
    }

    println!("We are back!");
    loop {
        for _ in 0..10000000 {
            unsafe { asm!("") };
        }
        print!("-");
    }
    //wfi_loop();
}

fn wfi_loop() -> ! {
    loop {
        unsafe { wfi() };
    }
}
