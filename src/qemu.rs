// Ripped from: https://github.com/andre-richter/qemu-exit/blob/master/src/riscv64.rs

use core::arch::asm;


const SIFIVE_TEST_ADDR: u64 = 0x100000;

// TODO: Support more exit codes
#[repr(u32)]
pub enum ExitCode {
    /// Equals `exit(0)`.
    Success = 0x5555,

    /// Equals `exit(1)`.
    Failed = 0x13333,

    Reset = 0x7777,
}

pub fn exit_qemu(code: ExitCode) {
    unsafe {
        asm!(
            "sw {0}, 0({1})",
            in(reg)code as u32, in(reg)SIFIVE_TEST_ADDR
        );

        loop {
            asm!("wfi", options(nomem, nostack));
        }
    }
}

pub fn exit_success() {
    exit_qemu(ExitCode::Success);
}

pub fn exit_failed() {
    exit_qemu(ExitCode::Failed);
}