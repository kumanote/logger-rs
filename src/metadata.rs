use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr, thread};

thread_local!(
    /// Thread-locally cached thread ID.
    static THREAD_ID: u64 = palaver::thread::gettid()
);

fn thread_id() -> u64 {
    THREAD_ID.with(|id| id.clone())
}

#[allow(dead_code)]
static HOSTNAME: Lazy<Option<String>> = Lazy::new(|| {
    hostname::get()
        .ok()
        .and_then(|name| name.into_string().ok())
});

fn hostname() -> Option<&'static str> {
    HOSTNAME.as_deref()
}

#[derive(Clone, Debug)]
pub struct Metadata {
    level: Level,
    target: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
    location: &'static str,
    thread_name: Option<String>,
    thread_id: u64,
    hostname: Option<&'static str>,
}

impl Metadata {
    pub fn new(
        level: Level,
        target: &'static str,
        module_path: &'static str,
        file: &'static str,
        line: u32,
        location: &'static str,
    ) -> Self {
        let thread_name = thread::current().name().map(ToOwned::to_owned);
        let thread_id = thread_id();
        let hostname = hostname();
        Self {
            level,
            target,
            module_path,
            file,
            line,
            location,
            thread_name,
            thread_id,
            hostname,
        }
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn target(&self) -> &'static str {
        self.target
    }

    pub fn module_path(&self) -> &'static str {
        self.module_path
    }

    pub fn file(&self) -> &'static str {
        self.file
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn location(&self) -> &'static str {
        self.location
    }

    pub fn thread_name(&self) -> Option<&str> {
        self.thread_name.as_deref()
    }

    pub fn thread_id(&self) -> u64 {
        self.thread_id
    }

    pub fn hostname(&self) -> Option<&'static str> {
        self.hostname
    }
}

static LOG_LEVEL_NAMES: &[&str] = &["CRASH", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

#[repr(usize)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Level {
    Crash = 0,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Level {
    fn from_usize(idx: usize) -> Option<Self> {
        let lvl = match idx {
            0 => Level::Crash,
            1 => Level::Error,
            2 => Level::Warn,
            3 => Level::Info,
            4 => Level::Debug,
            5 => Level::Trace,
            _ => return None,
        };

        Some(lvl)
    }
}

#[derive(Debug)]
pub struct LevelParseError;

impl FromStr for Level {
    type Err = LevelParseError;
    fn from_str(level: &str) -> Result<Level, Self::Err> {
        LOG_LEVEL_NAMES
            .iter()
            .position(|name| name.eq_ignore_ascii_case(level))
            .map(|idx| Level::from_usize(idx).unwrap())
            .ok_or(LevelParseError)
    }
}

impl fmt::Display for Level {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(LOG_LEVEL_NAMES[*self as usize])
    }
}
