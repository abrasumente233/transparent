use log::{Level, LevelFilter};

use crate::println;

pub fn init() {
    static CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger;
    log::set_logger(&CONSOLE_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);
}

struct ConsoleLogger;

impl log::Log for ConsoleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!(
                "\u{1B}[{}m {:>5}\u{1B}[0m {:<20} {}",
                level_to_color(record.level()),
                record.level(),
                record.module_path().unwrap(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

fn level_to_color(level: Level) -> u8 {
    match level {
        Level::Error => 31,
        Level::Warn => 93,
        Level::Info => 34,
        Level::Debug => 32,
        Level::Trace => 90,
    }
}
