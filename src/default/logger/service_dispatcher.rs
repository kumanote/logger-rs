use crate::default::logger_service::LoggerServiceEvent;
use crate::{AsyncEvent, Event, Filter, Metadata, StandardFilter};
use std::sync::mpsc::{self, SyncSender};

pub struct LoggerServiceDispatcher {
    pub(crate) filters: Vec<StandardFilter>,
    pub(crate) sender: SyncSender<LoggerServiceEvent>,
}

impl LoggerServiceDispatcher {
    pub(crate) fn enabled(&self, metadata: &Metadata) -> bool {
        for filter in &self.filters {
            if filter.enabled(metadata) {
                return true;
            }
        }
        false
    }

    pub(crate) fn record(&self, event: &Event) {
        let timestamp = event.timestamp().clone();
        let metadata = event.metadata().clone();
        let message = event.message().map(std::fmt::format);
        let keys_and_values = event.get_json_keys_and_values();
        let backtrace = event.backtrace().map(ToOwned::to_owned);
        let async_event = AsyncEvent::new(timestamp, metadata, message, keys_and_values, backtrace);
        if let Err(e) = self
            .sender
            .try_send(LoggerServiceEvent::LogEvent(async_event))
        {
            eprintln!("Failed to send structured log: {}", e);
        }
    }

    pub(crate) fn flush(&self) {
        let (oneshot_sender, oneshot_receiver) = mpsc::sync_channel(1);
        self.sender
            .send(LoggerServiceEvent::Flush(oneshot_sender))
            .unwrap();
        oneshot_receiver.recv().unwrap();
    }
}
