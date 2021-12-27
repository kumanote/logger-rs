use serde::Serialize;
use std::ops::Deref;
use std::{
    borrow::{Borrow, Cow},
    fmt,
};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct Key(Cow<'static, str>);

impl Key {
    pub fn new(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }

    pub fn new_owned(s: String) -> Self {
        Self(Cow::Owned(s))
    }

    pub fn as_str(&self) -> &'_ Self {
        self.borrow()
    }

    pub fn deref(&self) -> &str {
        self.0.deref()
    }
}

#[derive(Clone, Copy)]
pub enum Value<'v> {
    Debug(&'v (dyn fmt::Debug)),
    Display(&'v (dyn fmt::Display)),
    Serde(&'v (dyn erased_serde::Serialize)),
}

impl<'v> fmt::Debug for Value<'v> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Value::Debug(d) => fmt::Debug::fmt(d, f),
            Value::Display(d) => fmt::Display::fmt(d, f),
            Value::Serde(s) => {
                fmt::Debug::fmt(&serde_json::to_value(s).map_err(|_| fmt::Error)?, f)
            }
        }
    }
}

impl<'v> Value<'v> {
    pub fn from_serde<T: serde::Serialize>(value: &'v T) -> Self {
        Value::Serde(value)
    }

    pub fn from_debug<T: fmt::Debug>(value: &'v T) -> Self {
        Value::Debug(value)
    }

    pub fn from_display<T: fmt::Display>(value: &'v T) -> Self {
        Value::Display(value)
    }
}

#[derive(Clone, Debug)]
pub struct KeyValue<'v> {
    pub key: Key,
    pub value: Value<'v>,
}

impl<'v> KeyValue<'v> {
    pub fn new(key: &'static str, value: Value<'v>) -> Self {
        Self {
            key: Key::new(key),
            value,
        }
    }
}

impl<'v> Schema for KeyValue<'v> {
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.visit_pair(self.key.clone(), self.value)
    }
}

pub trait Schema {
    fn visit(&self, visitor: &mut dyn Visitor);
}

pub trait Visitor {
    fn visit_pair(&mut self, key: Key, value: Value<'_>);
}

impl<'a, 'b: 'a> Visitor for fmt::DebugMap<'a, 'b> {
    fn visit_pair(&mut self, key: Key, value: Value<'_>) {
        self.entry(&key, &value);
    }
}
