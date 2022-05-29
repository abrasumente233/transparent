const UART_BASE: u64 = 0x10000000;

pub(crate) fn init() {
    let word_length = (UART_BASE + 3) as *mut u8;
    let fifo_enable = (UART_BASE + 2) as *mut u8;
    let uart_enable = (UART_BASE + 1) as *mut u8;
    unsafe {
        // Set word length to 8-bits (LCR[1:0])
        word_length.write_volatile(0b11);

        // Enable FIFOs (FCR[0])
        fifo_enable.write_volatile(0b1);

        // Enable receiver interrupts,
        uart_enable.write_volatile(0b1);

        // @FIXME: Set baud rate to run on real hardware
        // other than QEMU
    }
}

pub(crate) fn get() -> i32 {
    let uart = UART_BASE as *mut u8;
    let ready = (UART_BASE + 5) as *mut u8;

    unsafe {
        if ready.read_volatile() & 1 == 0 {
            -1
        } else {
            uart.read_volatile() as i32
        }
    }
}
