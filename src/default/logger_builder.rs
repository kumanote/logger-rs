use super::*;
#[cfg(any(feature = "airbrake"))]
use crate::default::logger::AirbrakeLogger;
#[cfg(any(feature = "tcp"))]
use crate::default::logger::TcpLogger;
use crate::default::logger::{LocalFileLogger, SyncLoggerImpl};
use crate::default::logger_service::DefaultLoggerService;
use crate::formatter::*;
#[cfg(any(feature = "tcp"))]
use crate::writer::TcpWriter;
use crate::{writer::*, Level, StandardFilterBuilder};
use std::{
    env, path,
    sync::{mpsc, Arc},
    thread,
};

const RUST_LOG: &str = "RUST_LOG";
#[cfg(any(feature = "tcp"))]
const RUST_TCP_LOG: &str = "RUST_TCP_LOG";
/// Default size of log write channel, if the channel is full, logs will be dropped
pub const CHANNEL_SIZE: usize = 10000;

pub struct DefaultLoggerBuilder {
    file: Option<path::PathBuf>,
    level: Level,
    is_async: bool,
    channel_size: usize,
    #[cfg(any(feature = "tcp"))]
    tcp_level: Level,
    #[cfg(any(feature = "tcp"))]
    tcp_address: Option<String>,
    #[cfg(any(feature = "airbrake"))]
    airbrake_host: Option<String>,
    #[cfg(any(feature = "airbrake"))]
    airbrake_project_id: Option<String>,
    #[cfg(any(feature = "airbrake"))]
    airbrake_project_key: Option<String>,
    #[cfg(any(feature = "airbrake"))]
    airbrake_environment: Option<String>,
}

impl DefaultLoggerBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            file: None, // default print to console
            level: Level::Info,
            is_async: false,
            channel_size: CHANNEL_SIZE,
            #[cfg(any(feature = "tcp"))]
            tcp_level: Level::Info,
            #[cfg(any(feature = "tcp"))]
            tcp_address: None,
            #[cfg(any(feature = "airbrake"))]
            airbrake_host: None,
            #[cfg(any(feature = "airbrake"))]
            airbrake_project_id: None,
            #[cfg(any(feature = "airbrake"))]
            airbrake_project_key: None,
            #[cfg(any(feature = "airbrake"))]
            airbrake_environment: None,
        }
    }

    pub fn file<T: Into<path::PathBuf>>(&mut self, file: T) -> &mut Self {
        self.file = Some(file.into());
        self
    }

    pub fn level(&mut self, level: Level) -> &mut Self {
        self.level = level;
        self
    }

    // noinspection RsSelfConvention
    pub fn is_async(&mut self, is_async: bool) -> &mut Self {
        self.is_async = is_async;
        self
    }

    pub fn channel_size(&mut self, channel_size: usize) -> &mut Self {
        self.channel_size = channel_size;
        self
    }

    #[cfg(any(feature = "tcp"))]
    pub fn tcp_level(&mut self, tcp_level: Level) -> &mut Self {
        self.tcp_level = tcp_level;
        self
    }

    #[cfg(any(feature = "tcp"))]
    pub fn tcp_address(&mut self, tcp_address: String) -> &mut Self {
        self.tcp_address = Some(tcp_address);
        self
    }

    #[cfg(any(feature = "airbrake"))]
    pub fn airbrake_host(&mut self, airbrake_host: String) -> &mut Self {
        self.airbrake_host = Some(airbrake_host);
        self
    }

    #[cfg(any(feature = "airbrake"))]
    pub fn airbrake_project_id(&mut self, airbrake_project_id: String) -> &mut Self {
        self.airbrake_project_id = Some(airbrake_project_id);
        self
    }

    #[cfg(any(feature = "airbrake"))]
    pub fn airbrake_project_key(&mut self, airbrake_project_key: String) -> &mut Self {
        self.airbrake_project_key = Some(airbrake_project_key);
        self
    }

    #[cfg(any(feature = "airbrake"))]
    pub fn airbrake_environment(&mut self, airbrake_environment: String) -> &mut Self {
        self.airbrake_environment = Some(airbrake_environment);
        self
    }

    #[cfg(any(feature = "airbrake"))]
    pub fn airbrake_endpoint(&self) -> Option<String> {
        if self.airbrake_host.is_none() {
            return None;
        }
        if self.airbrake_project_id.is_none() {
            return None;
        }
        if self.airbrake_project_key.is_none() {
            return None;
        }
        let url = format!(
            "{}/api/v3/projects/{}/notices?key={}",
            self.airbrake_host.as_ref().unwrap(),
            self.airbrake_project_id.as_ref().unwrap(),
            self.airbrake_project_key.as_ref().unwrap(),
        );
        Some(url)
    }

    pub fn build(&mut self) -> Arc<DefaultLogger> {
        let filter = {
            let mut filter_builder = StandardFilterBuilder::new();
            if env::var(RUST_LOG).is_ok() {
                filter_builder.with_env(RUST_LOG);
            } else {
                filter_builder.filter_level(self.level.into());
            }
            filter_builder.build()
        };
        let logger = if self.is_async {
            let (sender, receiver) = mpsc::sync_channel(self.channel_size);
            let filters = vec![filter.clone()];
            let mut loggers = vec![];
            {
                let logger = if let Some(file_path) = &self.file {
                    ServiceLoggerImpl::LocalFile(LocalFileLogger {
                        printer: FileWriter::new(file_path.clone()),
                        filter,
                        formatter: StandardFormatter,
                    })
                } else {
                    ServiceLoggerImpl::LocalConsole(LocalConsoleLogger {
                        printer: StderrWriter,
                        filter,
                        formatter: StandardFormatter,
                    })
                };
                loggers.push(logger)
            }

            #[cfg(any(feature = "tcp"))]
            if let Some(tcp_address) = self.tcp_address.as_deref() {
                let tcp_filter = {
                    let mut filter_builder = StandardFilterBuilder::new();
                    if env::var(RUST_TCP_LOG).is_ok() {
                        filter_builder.with_env(RUST_TCP_LOG);
                    } else {
                        filter_builder.filter_level(self.tcp_level.into());
                    }
                    filter_builder.build()
                };
                let logger = ServiceLoggerImpl::Tcp(TcpLogger {
                    printer: TcpWriter::new(tcp_address.to_owned()),
                    filter: tcp_filter,
                    formatter: JsonFormatter,
                });
                loggers.push(logger);
            }
            #[cfg(any(feature = "airbrake"))]
            if let Some(airbrake_endpoint) = self.airbrake_endpoint() {
                let environment = self.airbrake_environment.clone();
                let logger = ServiceLoggerImpl::Airbrake(AirbrakeLogger::new(
                    airbrake_endpoint,
                    environment,
                ));
                loggers.push(logger);
            }
            let logger = LoggerServiceDispatcher { filters, sender };
            let service = DefaultLoggerService { receiver, loggers };
            thread::spawn(move || service.run());
            Arc::new(DefaultLogger::Async(logger))
        } else {
            let mut loggers = vec![];
            {
                let logger = if let Some(file_path) = &self.file {
                    SyncLoggerImpl::LocalFile(LocalFileLogger {
                        printer: FileWriter::new(file_path.clone()),
                        filter,
                        formatter: StandardFormatter,
                    })
                } else {
                    SyncLoggerImpl::LocalConsole(LocalConsoleLogger {
                        printer: StderrWriter,
                        filter,
                        formatter: StandardFormatter,
                    })
                };
                loggers.push(logger);
            }
            #[cfg(any(feature = "tcp"))]
            if let Some(_) = self.tcp_address.as_deref() {
                panic!("tcp logger is not supported for syncing mode.")
            }
            #[cfg(any(feature = "airbrake"))]
            if let Some(airbrake_endpoint) = self.airbrake_endpoint() {
                let environment = self.airbrake_environment.clone();
                let logger =
                    SyncLoggerImpl::Airbrake(AirbrakeLogger::new(airbrake_endpoint, environment));
                loggers.push(logger);
            }
            if loggers.len() > 1 {
                Arc::new(DefaultLogger::SyncMulti(loggers))
            } else {
                let logger = loggers.into_iter().next().unwrap();
                Arc::new(DefaultLogger::Sync(logger))
            }
        };
        crate::set_global_logger(logger.clone());
        logger
    }
}
