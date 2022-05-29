use riscv::register::time;
use sbi::legacy::set_timer;

pub(crate) fn init() {
    unsafe {
        riscv::register::sie::set_stimer();
    }
    set_next_timer();
}

pub(crate) fn set_next_timer() {
    set_timer(time::read64() + 10000);
}
