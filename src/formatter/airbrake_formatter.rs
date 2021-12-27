use super::{AsyncFormatter, Formatter};
use crate::{AsyncEvent, Event, Level};
use serde::{Serialize, Serializer};
use serde_json;
use std::{
    collections::HashMap,
    env,
    fmt::{self, Error},
};

pub struct AirbrakeFormatter {
    environment: Option<String>,
}

impl AirbrakeFormatter {
    pub fn new(environment: Option<String>) -> Self {
        Self { environment }
    }
}

impl Formatter for AirbrakeFormatter {
    fn format(&self, event: &Event) -> Result<String, Error> {
        let message = event.message().map(fmt::format).unwrap_or("".to_owned());
        let backtrace = event
            .backtrace()
            .map(|backtrace| parse_backtrace(backtrace));
        let error_info = ErrorInfo {
            type_: "".to_owned(),
            message,
            backtrace,
        };
        let severity = Some(event.metadata().level().into());
        let hostname = event.metadata().hostname().map(ToOwned::to_owned);
        let context = Context {
            notifier: Some(NotifierInfo::default()),
            environment: self.environment.clone(),
            severity,
            component: None,
            action: None,
            os: Some(env::consts::OS.to_owned()),
            hostname,
            language: None,
            version: Some(format!("{}", env!("CARGO_PKG_VERSION"))),
            url: None,
            user_agent: None,
            user_addr: None,
            remote_addr: None,
            root_directory: None,
            user: None,
            route: None,
            http_method: None,
        };
        let notice = Notice {
            errors: vec![error_info],
            context,
            environment: None,
            session: None,
            params: None,
        };
        Ok(serde_json::to_string(&notice).unwrap())
    }
}

impl AsyncFormatter for AirbrakeFormatter {
    fn format(&self, event: &AsyncEvent) -> Result<String, Error> {
        let message = event.message().unwrap_or("").to_owned();
        let backtrace = event
            .backtrace()
            .map(|backtrace| parse_backtrace(backtrace));
        let error_info = ErrorInfo {
            type_: "".to_owned(),
            message,
            backtrace,
        };
        let severity = Some(event.metadata().level().into());
        let hostname = event.metadata().hostname().map(ToOwned::to_owned);
        let context = Context {
            notifier: Some(NotifierInfo::default()),
            environment: self.environment.clone(),
            severity,
            component: None,
            action: None,
            os: Some(env::consts::OS.to_owned()),
            hostname,
            language: None,
            version: Some(format!("{}", env!("CARGO_PKG_VERSION"))),
            url: None,
            user_agent: None,
            user_addr: None,
            remote_addr: None,
            root_directory: None,
            user: None,
            route: None,
            http_method: None,
        };
        let notice = Notice {
            errors: vec![error_info],
            context,
            environment: None,
            session: None,
            params: None,
        };
        Ok(serde_json::to_string(&notice).unwrap())
    }
}

/// @see https://airbrake.io/docs/api/#create-notice-v3
#[derive(Debug, Serialize)]
pub struct Notice {
    pub errors: Vec<ErrorInfo>,
    pub context: Context,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct ErrorInfo {
    #[serde(rename = "type")]
    pub type_: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backtrace: Option<Vec<BacktraceInfo>>,
}

fn parse_backtrace(s: &str) -> Vec<BacktraceInfo> {
    let backtraces: Vec<&str> = s
        .split("\n")
        .into_iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.trim())
        .collect();
    let mut backtrace_infos = vec![];
    let mut item = BacktraceInfo::default();
    let mut backtrace_iter = backtraces.into_iter();
    loop {
        if let Some(t) = backtrace_iter.next() {
            if t.starts_with("at ") {
                let position_part = &t[3..];
                let position_info: Vec<&str> = position_part.split(":").into_iter().collect();
                if position_info.len() > 0 {
                    item.file = Some(position_info[0].to_owned())
                }
                if position_info.len() > 1 {
                    if let Ok(l) = position_info[1].parse::<usize>() {
                        item.line = Some(l)
                    }
                }
                if position_info.len() > 2 {
                    if let Ok(c) = position_info[2].parse::<usize>() {
                        item.column = Some(c)
                    }
                }
                backtrace_infos.push(item.clone());
                item = BacktraceInfo::default();
            } else {
                item = BacktraceInfo::default();
                item.function = Some(t.to_owned())
            }
        } else {
            if !item.is_empty() {
                backtrace_infos.push(item.clone())
            }
            break;
        }
    }
    backtrace_infos
}

#[derive(Clone, Debug, Serialize)]
pub struct BacktraceInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<HashMap<String, String>>,
}

impl BacktraceInfo {
    pub fn is_empty(&self) -> bool {
        self.file.is_none()
            && self.function.is_none()
            && self.line.is_none()
            && self.column.is_none()
            && self.code.is_none()
    }
}

impl Default for BacktraceInfo {
    fn default() -> Self {
        Self {
            file: None,
            function: None,
            line: None,
            column: None,
            code: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Context {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifier: Option<NotifierInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_severity"
    )]
    pub severity: Option<Severity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "userAgent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(rename = "userAddr")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_addr: Option<String>,
    #[serde(rename = "remoteAddr")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_addr: Option<String>,
    #[serde(rename = "rootDirectory")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(rename = "httpMethod")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_method: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NotifierInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl Default for NotifierInfo {
    fn default() -> Self {
        Self {
            name: Some(format!("{}", env!("CARGO_PKG_NAME"))),
            version: Some(format!("{}", env!("CARGO_PKG_VERSION"))),
            url: Some(format!("{}", env!("CARGO_PKG_REPOSITORY"))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Severity {
    Debug,
    Info,
    #[allow(dead_code)]
    Notice,
    Warning,
    Error,
    Critical,
    #[allow(dead_code)]
    Alert,
    #[allow(dead_code)]
    Emergency,
    Invalid,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Debug => write!(f, "debug"),
            Severity::Info => write!(f, "info"),
            Severity::Notice => write!(f, "notice"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
            Severity::Critical => write!(f, "critical"),
            Severity::Alert => write!(f, "alert"),
            Severity::Emergency => write!(f, "emergency"),
            Severity::Invalid => write!(f, "invalid"),
        }
    }
}

impl From<Level> for Severity {
    fn from(level: Level) -> Self {
        match level {
            Level::Crash => Self::Critical,
            Level::Error => Self::Error,
            Level::Warn => Self::Warning,
            Level::Info => Self::Info,
            Level::Debug => Self::Debug,
            _ => panic!("unsupported level: {}", level),
        }
    }
}

fn serialize_severity<S>(value: &Option<Severity>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let severity = if let Some(severity) = value {
        severity
    } else {
        &Severity::Invalid
    };
    serializer.serialize_str(severity.to_string().as_str())
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}
