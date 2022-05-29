use riscv::register::time;
use sbi::legacy::set_timer;

//const QEMU_FREQ: usize = 1_000_000;
const QEMU_FREQ: usize = 12_500_000;
const TICKS_PER_SEC: usize = 10;

pub(crate) fn init() {
    unsafe {
        riscv::register::sie::set_stimer();
    }
    set_next_timer();
}

pub(crate) fn set_next_timer() {
    set_timer(time::read64() + (QEMU_FREQ / TICKS_PER_SEC) as u64);
}
