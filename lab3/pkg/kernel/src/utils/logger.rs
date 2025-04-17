use log::{Level, LevelFilter, Metadata, Record};

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();

    // FIXME: Configure the logger
    log::set_max_level(LevelFilter::Trace);
    info!("Logger Initialized.");
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        _metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        // FIXME: Implement the logger with serial output
        let color_code = match record.level() {
            Level::Error => "\x1b[31m", // red
            Level::Warn => "\x1b[33m",  // yellow
            Level::Info => "\x1b[32m",  // green
            Level::Debug => "\x1b[36m", // cyan
            Level::Trace => "\x1b[90m", // black
        };
        let reset_code = "\x1b[0m";
        println!(
            "{}[{}]{} {}",
            color_code,
            record.level(),
            reset_code,
            record.args()
        );
    }

    fn flush(&self) {}
}
