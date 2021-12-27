use crate::{AsyncEvent, Event, Metadata};

pub trait Logger: Sync + Send + 'static {
    fn enabled(&self, metadata: &Metadata) -> bool;
    fn record(&self, event: &Event);
    fn flush(&self);
}

pub trait AsyncLogger: Logger {
    fn record(&self, event: &AsyncEvent);
}
