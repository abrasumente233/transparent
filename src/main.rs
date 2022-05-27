
#![no_std]
#![no_main]

mod assembly;
mod console;
mod panic;

#[no_mangle]
pub fn main() -> ! {
    println!("Hello, world!");
    panic!("test panic!");
    loop {} 
}
