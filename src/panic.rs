use core::panic::PanicInfo;
use sbi::legacy::shutdown;

use crate::println;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("error: {}", _info);
    shutdown();
}
