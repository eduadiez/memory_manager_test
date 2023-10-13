// logger.rs
extern crate slog;

use once_cell::sync::Lazy;
use slog::{o, Drain, Logger};
use std::sync::Mutex;

static LOGGER: Lazy<Logger> = Lazy::new(|| {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();
    slog::Logger::root(drain, o!())
});

pub fn get_logger() -> &'static Logger {
    &LOGGER
}
