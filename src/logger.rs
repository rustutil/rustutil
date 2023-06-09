use colored::Colorize;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct ColoredLogger;

fn level_to_color(level: Level) -> colored::Color {
    match level {
        Level::Error => colored::Color::Red,
        Level::Warn => colored::Color::Yellow,
        Level::Info => colored::Color::Blue,
        Level::Debug => colored::Color::Green,
        Level::Trace => colored::Color::White,
    }
}

impl log::Log for ColoredLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "- {} {}",
                record
                    .level()
                    .to_string()
                    .to_lowercase()
                    .color(level_to_color(record.level())),
                // .bold(),
                // ":".bold(),
                // record.target().green().bold(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

static LOGGER: ColoredLogger = ColoredLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}
