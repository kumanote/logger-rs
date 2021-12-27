pub mod prelude {
    #[cfg(any(feature = "airbrake"))]
    pub use crate::writer::HttpWriter;
    #[cfg(any(feature = "tcp"))]
    pub use crate::writer::TcpWriter;
    pub use crate::{
        crash, debug, error, info, trace, warn,
        writer::{FileWriter, StderrWriter},
    };
}
pub mod default;

mod crash_handler;
mod event;
mod filter;
mod formatter;
mod kv;
mod logger;
mod metadata;
mod scope;
mod writer;

pub use crash_handler::setup_panic_logger;
pub use event::{AsyncEvent, Event};
pub use filter::{Filter, StandardFilter, StandardFilterBuilder};
#[cfg(any(feature = "airbrake"))]
pub use formatter::AirbrakeFormatter;
pub use formatter::{AsyncFormatter, Formatter, JsonFormatter, StandardFormatter};
pub use kv::{Key, KeyValue, Schema, Value, Visitor};
pub use logger::{AsyncLogger, Logger};
pub use metadata::{Level, Metadata};
pub use writer::Writer;

use once_cell::sync::OnceCell;
use std::sync::Arc;

pub static LOGGER: OnceCell<Arc<dyn Logger>> = OnceCell::new();

pub fn set_global_logger(logger: Arc<dyn Logger>) {
    if LOGGER.set(logger).is_err() {
        eprintln!("Global logger has already been set");
    }
}

pub fn flush() {
    if let Some(logger) = LOGGER.get() {
        logger.flush()
    }
}
