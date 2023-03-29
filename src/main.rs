#![no_std]
#![no_main]
#![feature(
    naked_functions,
    core_intrinsics,
    asm_const,
    asm_sym,
    fn_align,
    alloc_error_handler,
    custom_test_frameworks
)]
#![test_runner(testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use ::log::info;
use riscv::asm::wfi;
use sbi::hart_state_management::hart_status;

use crate::{block::BLK, fat32::Fat32};

mod addr;
mod align;
mod allocator;
mod assembly;
mod block;
mod console;
mod device;
mod fat32;
mod log;
mod memory;
mod panic;
mod plic;
mod qemu;
mod testing;
mod timer;
mod trap;
mod uart;

#[no_mangle]
pub fn rust_start(hartid: usize, device_tree_paddr: usize) -> ! {
    #[cfg(test)]
    {
        test_main();
        loop {}
    }

    #[cfg(not(test))]
    main(hartid, device_tree_paddr);
}

#[no_mangle]
pub fn main(_hartid: usize, device_tree_paddr: usize) -> ! {
    log::init();
    trap::init();
    uart::init();
    plic::init();
    timer::init();
    allocator::init();
    device::init(device_tree_paddr);
    memory::init();

    let pt = unsafe { memory::active_level_3_table() };
    for pte in pt.iter() {
        if pte.is_valid() {
            info!("{:X?}", pte);
        }
    }

    let addresses: [u64; 3] = [
        0x80000000,
        0xc000000,
        0x82000000
    ];

    for address in addresses {
        let addr = addr::VirtAddr::new_truncate(address);
        let paddr = unsafe { memory::translate_addr(addr) };
        info!("{:x?} -> {:x?}", addr, paddr); 
    }

    let blk = unsafe { BLK.take().unwrap() };
    let mut fat32 = Fat32::new(blk);
    fat32.check_fs();
    fat32.ls_rootdir();

    /*
    let mut buf = vec![0; 512];
    unsafe {
        BLK.as_mut().unwrap().read(0, &mut buf).unwrap();
    }
    let x = buf.as_slice();
    println!("{:X?}", x);
    */

    // let mut v = vec![];
    // for i in 1..512 {
    //     v.push(i);
    // }
    // for i in v {
    //     trace!("{}", i);
    // }

    info!("Hello, world!");
    info!("hart #0 status: {:?}", hart_status(0));

    // let elf = include_bytes!("../user/busy.elf");
    // let file = ElfBytes::<NativeEndian>::minimal_parse(elf).unwrap();
    // assert_eq!(file.ehdr.e_machine, 0xf3); // is RISC-V
    // assert_eq!(file.ehdr.e_type, 2); // is executable

    // let first_load_ph = file
    //     .segments()
    //     .unwrap()
    //     .iter()
    //     .find(|ph| ph.p_type == 1)
    //     .unwrap(); // PT_LOAD

    // let code = file.segment_data(&first_load_ph).unwrap();
    // info!("code: {:0>2x?}", code);

    /*
    unsafe {
        asm!("addi a0, a0, 0");
        asm!("ebreak");
        asm!("addi a0, a0, 0");
    }
    */

    info!("We are back!");
    wfi_loop();
}

fn wfi_loop() -> ! {
    loop {
        unsafe { wfi() };
    }
}
