mod local_console;
pub use local_console::LocalConsoleLogger;

mod local_file;
pub use local_file::LocalFileLogger;

#[cfg(any(feature = "tcp"))]
mod tcp;
#[cfg(any(feature = "tcp"))]
pub use tcp::TcpLogger;

#[cfg(any(feature = "airbrake"))]
mod airbrake;
#[cfg(any(feature = "airbrake"))]
pub use airbrake::AirbrakeLogger;

mod service_dispatcher;
pub use service_dispatcher::LoggerServiceDispatcher;

use crate::{Event, Logger, Metadata};

pub enum DefaultLogger {
    Sync(SyncLoggerImpl),
    SyncMulti(Vec<SyncLoggerImpl>),
    Async(LoggerServiceDispatcher),
}

impl Logger for DefaultLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        match &self {
            DefaultLogger::Sync(logger) => logger.enabled(metadata),
            DefaultLogger::SyncMulti(loggers) => {
                for logger in loggers {
                    if logger.enabled(metadata) {
                        return true;
                    }
                }
                false
            }
            DefaultLogger::Async(inner) => inner.enabled(metadata),
        }
    }

    fn record(&self, event: &Event) {
        match self {
            DefaultLogger::Sync(logger) => Logger::record(logger, event),
            DefaultLogger::SyncMulti(loggers) => {
                for logger in loggers {
                    if logger.enabled(&event.metadata()) {
                        Logger::record(logger, event);
                    }
                }
            }
            DefaultLogger::Async(inner) => inner.record(event),
        }
    }

    fn flush(&self) {
        match self {
            DefaultLogger::Sync(logger) => logger.flush(),
            DefaultLogger::SyncMulti(loggers) => {
                for logger in loggers {
                    logger.flush();
                }
            }
            DefaultLogger::Async(inner) => inner.flush(),
        }
    }
}

pub enum SyncLoggerImpl {
    LocalConsole(LocalConsoleLogger),
    LocalFile(LocalFileLogger),
    #[cfg(any(feature = "tcp"))]
    Tcp(TcpLogger),
    #[cfg(any(feature = "airbrake"))]
    Airbrake(AirbrakeLogger),
}

impl Logger for SyncLoggerImpl {
    fn enabled(&self, metadata: &Metadata) -> bool {
        match &self {
            SyncLoggerImpl::LocalConsole(inner) => inner.enabled(metadata),
            SyncLoggerImpl::LocalFile(inner) => inner.enabled(metadata),
            #[cfg(any(feature = "tcp"))]
            SyncLoggerImpl::Tcp(inner) => inner.enabled(metadata),
            #[cfg(any(feature = "airbrake"))]
            SyncLoggerImpl::Airbrake(inner) => inner.enabled(metadata),
        }
    }

    fn record(&self, event: &Event) {
        match self {
            SyncLoggerImpl::LocalConsole(inner) => Logger::record(inner, event),
            SyncLoggerImpl::LocalFile(inner) => Logger::record(inner, event),
            #[cfg(any(feature = "tcp"))]
            SyncLoggerImpl::Tcp(inner) => Logger::record(inner, event),
            #[cfg(any(feature = "airbrake"))]
            SyncLoggerImpl::Airbrake(inner) => Logger::record(inner, event),
        }
    }

    fn flush(&self) {
        match self {
            SyncLoggerImpl::LocalConsole(inner) => inner.flush(),
            SyncLoggerImpl::LocalFile(inner) => inner.flush(),
            #[cfg(any(feature = "tcp"))]
            SyncLoggerImpl::Tcp(inner) => inner.flush(),
            #[cfg(any(feature = "airbrake"))]
            SyncLoggerImpl::Airbrake(inner) => inner.flush(),
        }
    }
}
