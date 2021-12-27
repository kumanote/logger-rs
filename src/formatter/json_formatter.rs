use super::{AsyncFormatter, Formatter};
use crate::{AsyncEvent, Event};
use serde_json::json;
use std::fmt::{self, Error, Write};

pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format(&self, event: &Event) -> Result<String, Error> {
        let mut w = String::new();
        let hostname = event.metadata().hostname().unwrap_or("");
        let timestamp = event.timestamp().timestamp();
        let thread_name = event.metadata().thread_name().unwrap_or("");
        let thread_id = event.metadata().thread_id();
        let level = event.metadata().level();
        let location = event.metadata().location();
        let message = event.message().map(fmt::format).unwrap_or("".to_owned());
        let data = event.get_json_keys_and_values();
        let backtrace = event.backtrace().unwrap_or("");
        let log_line = json!({
            "hostname": hostname,
            "timestamp": timestamp,
            "thread_name": thread_name,
            "thread_id": thread_id,
            "level": level,
            "location": location,
            "message": message,
            "data": data,
            "backtrace": backtrace,
        });
        write!(w, "{}", log_line)?;
        Ok(w)
    }
}

impl AsyncFormatter for JsonFormatter {
    fn format(&self, event: &AsyncEvent) -> Result<String, Error> {
        let mut w = String::new();
        let hostname = event.metadata().hostname().unwrap_or("");
        let timestamp = event.timestamp().timestamp();
        let thread_name = event.metadata().thread_name().unwrap_or("");
        let thread_id = event.metadata().thread_id();
        let level = event.metadata().level();
        let location = event.metadata().location();
        let message = event.message().unwrap_or("");
        let data = event.keys_and_values();
        let backtrace = event.backtrace().unwrap_or("");
        let log_line = json!({
            "hostname": hostname,
            "timestamp": timestamp,
            "thread_name": thread_name,
            "thread_id": thread_id,
            "level": level,
            "location": location,
            "message": message,
            "data": data,
            "backtrace": backtrace,
        });
        write!(w, "{}", log_line)?;
        Ok(w)
    }
}
