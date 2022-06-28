use core::fmt;

use sbi::legacy::console_putchar;
use spin::Mutex;

use crate::trap::without_interrupts;

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {});
//pub static mut WRITER: Writer = Writer {};

pub struct Writer;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &b in s.as_bytes() {
            console_putchar(b);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    without_interrupts(|| {
        // unsafe { WRITER.write_fmt(args).unwrap(); }
        WRITER.lock().write_fmt(args).unwrap();
    });
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

macro_rules! impl_color {
    ($color_name:ident, $ansi_code:literal) => {
        pub struct $color_name(pub &'static str);

        impl core::fmt::Display for $color_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                //write!(f, "\x1b[32m{}\x1b[0m", self.0)
                write!(f, concat!("\x1b[", $ansi_code, "m{}\x1b[0m"), self.0)
            }
        }
    };
}

impl_color!(Red, 31);
impl_color!(Green, 32);
impl_color!(Yellow, 33);
impl_color!(Blue, 34);

#[test_case]
fn test_println_output() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}
