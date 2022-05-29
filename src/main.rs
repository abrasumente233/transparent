#![no_std]
#![no_main]
#![feature(naked_functions, core_intrinsics, asm_const, asm_sym, fn_align)]

use core::arch::asm;

mod assembly;
mod console;
mod panic;
mod trap;

#[no_mangle]
pub fn main() -> ! {
    trap::init();

    println!("Hello, world!");

    unsafe {
        asm!("addi a0, a0, 0");
        asm!("ebreak");
        asm!("addi a0, a0, 0");
    }
    println!("We are back!");
    loop {} 
}
