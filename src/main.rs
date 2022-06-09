#![no_std]
#![no_main]
#![feature(
    naked_functions,
    core_intrinsics,
    asm_const,
    asm_sym,
    fn_align,
    alloc_error_handler
)]

extern crate alloc;

use core::arch::asm;

use alloc::vec;
use riscv::asm::wfi;
use sbi::hart_state_management::hart_status;

mod allocator;
mod assembly;
mod console;
mod device;
mod panic;
mod plic;
mod timer;
mod trap;
mod uart;

#[no_mangle]
pub fn main(_hartid: usize, device_tree_paddr: usize) -> ! {
    trap::init();
    uart::init();
    plic::init();
    timer::init();
    allocator::init();
    device::init(device_tree_paddr);

    let mut v = vec![];
    for i in 1..512 {
        v.push(i);
    }
    for i in v {
        println!("{}", i);
    }

    println!("Hello, world!");
    println!("hart #0 status: {:?}", hart_status(0));

    unsafe {
        asm!("addi a0, a0, 0");
        asm!("ebreak");
        asm!("addi a0, a0, 0");
    }

    println!("We are back!");
    wfi_loop();
}

fn wfi_loop() -> ! {
    loop {
        unsafe { wfi() };
    }
}
