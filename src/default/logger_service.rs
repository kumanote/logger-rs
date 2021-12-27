use super::logger::*;
use crate::{AsyncEvent, AsyncLogger, Event, Logger, Metadata};
use std::sync::mpsc::{Receiver, SyncSender};

pub enum LoggerServiceEvent {
    LogEvent(AsyncEvent),
    Flush(SyncSender<()>),
}

pub enum ServiceLoggerImpl {
    LocalConsole(LocalConsoleLogger),
    LocalFile(LocalFileLogger),
    #[cfg(any(feature = "tcp"))]
    Tcp(TcpLogger),
    #[cfg(any(feature = "airbrake"))]
    Airbrake(AirbrakeLogger),
}

impl Logger for ServiceLoggerImpl {
    fn enabled(&self, metadata: &Metadata) -> bool {
        match &self {
            ServiceLoggerImpl::LocalConsole(inner) => inner.enabled(metadata),
            ServiceLoggerImpl::LocalFile(inner) => inner.enabled(metadata),
            #[cfg(any(feature = "tcp"))]
            ServiceLoggerImpl::Tcp(inner) => inner.enabled(metadata),
            #[cfg(any(feature = "airbrake"))]
            ServiceLoggerImpl::Airbrake(inner) => inner.enabled(metadata),
        }
    }

    fn record(&self, event: &Event) {
        match self {
            ServiceLoggerImpl::LocalConsole(inner) => Logger::record(inner, event),
            ServiceLoggerImpl::LocalFile(inner) => Logger::record(inner, event),
            #[cfg(any(feature = "tcp"))]
            ServiceLoggerImpl::Tcp(inner) => Logger::record(inner, event),
            #[cfg(any(feature = "airbrake"))]
            ServiceLoggerImpl::Airbrake(inner) => Logger::record(inner, event),
        }
    }

    fn flush(&self) {
        match self {
            ServiceLoggerImpl::LocalConsole(inner) => inner.flush(),
            ServiceLoggerImpl::LocalFile(inner) => inner.flush(),
            #[cfg(any(feature = "tcp"))]
            ServiceLoggerImpl::Tcp(inner) => inner.flush(),
            #[cfg(any(feature = "airbrake"))]
            ServiceLoggerImpl::Airbrake(inner) => inner.flush(),
        }
    }
}

impl AsyncLogger for ServiceLoggerImpl {
    fn record(&self, event: &AsyncEvent) {
        match self {
            ServiceLoggerImpl::LocalConsole(inner) => AsyncLogger::record(inner, event),
            ServiceLoggerImpl::LocalFile(inner) => AsyncLogger::record(inner, event),
            #[cfg(any(feature = "tcp"))]
            ServiceLoggerImpl::Tcp(inner) => AsyncLogger::record(inner, event),
            #[cfg(any(feature = "airbrake"))]
            ServiceLoggerImpl::Airbrake(inner) => AsyncLogger::record(inner, event),
        }
    }
}

pub struct DefaultLoggerService {
    pub(crate) receiver: Receiver<LoggerServiceEvent>,
    pub(crate) loggers: Vec<ServiceLoggerImpl>,
}

impl DefaultLoggerService {
    pub(crate) fn run(self) {
        // listen event to come...
        for event in self.receiver {
            match event {
                LoggerServiceEvent::LogEvent(event) => {
                    for logger in &self.loggers {
                        if logger.enabled(&event.metadata()) {
                            AsyncLogger::record(logger, &event)
                        }
                    }
                }
                LoggerServiceEvent::Flush(sender) => {
                    for logger in &self.loggers {
                        logger.flush()
                    }
                    // notify sender that the logger service has just handled flush message.
                    let _ = sender.send(());
                }
            }
        }
    }
}
