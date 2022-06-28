use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    use log::error;
    use sbi::legacy::shutdown;
    error!("{}", info);
    shutdown();
}

#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    use crate::console::Red;
    use crate::println;
    use crate::qemu::{exit_qemu, ExitCode};

    println!("{}\n", Red("[failed]"));
    println!("Error: {}\n", info);
    exit_qemu(ExitCode::Failed);
    loop {}
}
