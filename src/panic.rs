use core::panic::PanicInfo;
use log::error;
use sbi::legacy::shutdown;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    shutdown();
}
