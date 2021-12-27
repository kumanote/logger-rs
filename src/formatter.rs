use crate::{AsyncEvent, Event};
use std::fmt;

#[allow(dead_code)]
#[cfg(windows)]
const NEWLINE: &'static str = "\r\n";

#[allow(dead_code)]
#[cfg(not(windows))]
const NEWLINE: &str = "\n";

pub trait Formatter: Send + Sync {
    fn format(&self, event: &Event) -> Result<String, fmt::Error>;
}

pub trait AsyncFormatter: Formatter {
    fn format(&self, event: &AsyncEvent) -> Result<String, fmt::Error>;
}

mod standard_formatter;
pub use standard_formatter::StandardFormatter;

mod json_formatter;
pub use json_formatter::JsonFormatter;

#[cfg(any(feature = "airbrake"))]
mod airbrake_formatter;
#[cfg(any(feature = "airbrake"))]
pub use airbrake_formatter::AirbrakeFormatter;
