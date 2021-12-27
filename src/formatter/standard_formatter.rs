use super::{AsyncFormatter, Formatter, NEWLINE};
use crate::{AsyncEvent, Event};
use chrono::SecondsFormat;
use std::fmt::{self, Error, Write};

pub struct StandardFormatter;

impl Formatter for StandardFormatter {
    fn format(&self, event: &Event) -> Result<String, Error> {
        let mut w = String::new();

        // timestamp in fixed 6 digits format
        let timestamp = event
            .timestamp()
            .to_rfc3339_opts(SecondsFormat::Micros, true);
        write!(w, "{}", timestamp)?;

        // thread name
        if let Some(thread_name) = event.metadata().thread_name() {
            write!(w, " [{}]", thread_name)?;
        }

        // thread id
        write!(w, " {}", event.metadata().thread_id())?;

        // level & place
        write!(
            w,
            " {} {}",
            event.metadata().level(),
            event.metadata().location()
        )?;

        // message
        if let Some(message) = event.message().map(fmt::format) {
            write!(w, " {}", message)?;
        }

        // data
        let data = event.get_json_keys_and_values();
        if !data.is_empty() {
            write!(w, " {}", serde_json::to_string(&data).unwrap())?;
        }

        // backtrace
        if let Some(backtrace) = event.backtrace() {
            write!(w, "{}{}", NEWLINE, backtrace)?;
        }
        Ok(w)
    }
}

impl AsyncFormatter for StandardFormatter {
    fn format(&self, event: &AsyncEvent) -> Result<String, Error> {
        let mut w = String::new();

        // timestamp in fixed 6 digits format
        let timestamp = event
            .timestamp()
            .to_rfc3339_opts(SecondsFormat::Micros, true);
        write!(w, "{}", timestamp)?;

        // thread name
        if let Some(thread_name) = event.metadata().thread_name() {
            write!(w, " [{}]", thread_name)?;
        }

        // thread id
        write!(w, " {}", event.metadata().thread_id())?;

        // level & place
        write!(
            w,
            " {} {}",
            event.metadata().level(),
            event.metadata().location()
        )?;

        // message
        if let Some(message) = event.message() {
            write!(w, " {}", message)?;
        }

        // data
        let data = event.keys_and_values();
        if !data.is_empty() {
            write!(w, " {}", serde_json::to_string(data).unwrap())?;
        }

        // backtrace
        if let Some(backtrace) = event.backtrace() {
            write!(w, "{}{}", NEWLINE, backtrace)?;
        }
        Ok(w)
    }
}
