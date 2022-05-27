use core::fmt;

use sbi::legacy::console_putchar;
use spin::Mutex;

pub(crate) static WRITER: Mutex<Writer> = Mutex::new(Writer {});

pub(crate) struct Writer;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &b in s.as_bytes() {
            console_putchar(b);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub(crate) fn _print(args: fmt::Arguments) {
    use fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}