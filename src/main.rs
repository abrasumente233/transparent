#![no_std]
#![no_main]
#![feature(naked_functions, core_intrinsics, asm_const)]

mod assembly;
mod console;
mod panic;
mod trap;

#[no_mangle]
pub fn main() -> ! {
    println!("Hello, world!");
    loop {} 
}
