use crate::{Key, Level, Metadata, Schema, Value, Visitor};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug)]
pub struct Event<'a> {
    timestamp: DateTime<Utc>,
    metadata: &'a Metadata,
    message: Option<fmt::Arguments<'a>>,
    keys_and_values: KeysAndValues<'a>,
    backtrace: Option<String>,
}

impl<'a> Event<'a> {
    pub fn new(
        metadata: &'a Metadata,
        message: Option<fmt::Arguments<'a>>,
        keys_and_values: &'a [&'a dyn Schema],
    ) -> Self {
        let timestamp = Utc::now();
        let backtrace = match metadata.level() {
            Level::Crash => {
                let mut backtrace = backtrace::Backtrace::new();
                let mut frames = backtrace.frames().to_vec();
                if frames.len() > 5 {
                    frames.drain(0..5); // Remove the first 5 unnecessary frames to simplify backtrace
                }
                backtrace = frames.into();
                Some(format!("{:?}", backtrace))
            }
            _ => None,
        };
        Self {
            timestamp,
            metadata,
            message,
            keys_and_values: KeysAndValues(keys_and_values),
            backtrace,
        }
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn metadata(&self) -> &'a Metadata {
        self.metadata
    }

    pub fn message(&self) -> Option<fmt::Arguments<'a>> {
        self.message
    }

    pub fn keys_and_values(&self) -> &'a [&'a dyn Schema] {
        self.keys_and_values.0
    }

    pub fn get_json_keys_and_values(&self) -> BTreeMap<Key, serde_json::Value> {
        let mut data = BTreeMap::new();
        for schema in self.keys_and_values() {
            schema.visit(&mut JsonVisitor(&mut data));
        }
        data
    }

    pub fn backtrace(&self) -> Option<&str> {
        self.backtrace.as_deref()
    }
}

#[derive(Debug)]
pub struct AsyncEvent {
    timestamp: DateTime<Utc>,
    metadata: Metadata,
    message: Option<String>,
    keys_and_values: BTreeMap<Key, serde_json::Value>,
    backtrace: Option<String>,
}

impl AsyncEvent {
    pub fn new(
        timestamp: DateTime<Utc>,
        metadata: Metadata,
        message: Option<String>,
        keys_and_values: BTreeMap<Key, serde_json::Value>,
        backtrace: Option<String>,
    ) -> Self {
        Self {
            timestamp,
            metadata,
            message,
            keys_and_values,
            backtrace,
        }
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn keys_and_values(&self) -> &BTreeMap<Key, serde_json::Value> {
        &self.keys_and_values
    }

    pub fn backtrace(&self) -> Option<&str> {
        self.backtrace.as_deref()
    }
}

#[derive(Clone)]
struct KeysAndValues<'a>(&'a [&'a dyn Schema]);

impl<'a> fmt::Debug for KeysAndValues<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut visitor = f.debug_map();
        for key_value in self.0 {
            key_value.visit(&mut visitor);
        }
        visitor.finish()
    }
}

struct JsonVisitor<'a>(&'a mut BTreeMap<Key, serde_json::Value>);

impl<'a> Visitor for JsonVisitor<'a> {
    fn visit_pair(&mut self, key: Key, value: Value<'_>) {
        let v = match value {
            Value::Debug(d) => serde_json::Value::String(format!("{:?}", d)),
            Value::Display(d) => serde_json::Value::String(d.to_string()),
            Value::Serde(s) => match serde_json::to_value(s) {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("error serializing structured log: {}", e);
                    return;
                }
            },
        };

        self.0.insert(key, v);
    }
}
