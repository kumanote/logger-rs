use crate::{Level, Metadata};
use std::{env, str::FromStr};

pub trait Filter: Send + Sync {
    fn enabled(&self, metadata: &Metadata) -> bool;
}

#[derive(Debug, Clone)]
pub struct StandardFilter {
    directives: Vec<Directive>,
}

impl Filter for StandardFilter {
    fn enabled(&self, metadata: &Metadata) -> bool {
        for directive in self.directives.iter().rev() {
            match &directive.name {
                Some(name) if !metadata.module_path().starts_with(name) => {}
                Some(..) | None => return LevelFilter::from(metadata.level()) <= directive.level,
            }
        }
        false
    }
}

impl StandardFilter {
    pub fn builder() -> StandardFilterBuilder {
        StandardFilterBuilder::new()
    }
}

#[derive(Default, Debug)]
pub struct StandardFilterBuilder {
    directives: Vec<Directive>,
}

impl StandardFilterBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_env(&mut self, env: &str) -> &mut Self {
        if let Ok(s) = env::var(env) {
            self.parse(&s);
        }
        self
    }

    pub fn filter_module(&mut self, module: &str, level: LevelFilter) -> &mut Self {
        self.filter(Some(module), level)
    }

    pub fn filter_level(&mut self, level: LevelFilter) -> &mut Self {
        self.filter(None, level)
    }

    pub fn filter(&mut self, module: Option<&str>, level: LevelFilter) -> &mut Self {
        self.directives.push(Directive::new(module, level));
        self
    }

    pub fn parse(&mut self, filters: &str) -> &mut Self {
        self.directives.extend(
            filters
                .split(',')
                .map(Directive::from_str)
                .filter_map(Result::ok),
        );
        self
    }

    pub fn build(&mut self) -> StandardFilter {
        if self.directives.is_empty() {
            // Add the default filter if none exist
            self.filter_level(LevelFilter::Error);
        } else {
            // Sort the directives by length of their name, this allows a
            // little more efficient lookup at runtime.
            self.directives.sort_by(|a, b| {
                let alen = a.name.as_ref().map(|a| a.len()).unwrap_or(0);
                let blen = b.name.as_ref().map(|b| b.len()).unwrap_or(0);
                alen.cmp(&blen)
            });
        }

        StandardFilter {
            directives: ::std::mem::take(&mut self.directives),
        }
    }
}

pub struct FilterParseError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LevelFilter {
    Off,
    Crash,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LevelFilter {
    pub fn max() -> Self {
        LevelFilter::Trace
    }
}

impl FromStr for LevelFilter {
    type Err = FilterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let level = if s.eq_ignore_ascii_case("OFF") {
            LevelFilter::Off
        } else {
            s.parse::<Level>().map_err(|_| FilterParseError)?.into()
        };
        Ok(level)
    }
}

impl From<Level> for LevelFilter {
    fn from(level: Level) -> Self {
        match level {
            Level::Crash => LevelFilter::Crash,
            Level::Error => LevelFilter::Error,
            Level::Warn => LevelFilter::Warn,
            Level::Info => LevelFilter::Info,
            Level::Debug => LevelFilter::Debug,
            Level::Trace => LevelFilter::Trace,
        }
    }
}

#[derive(Debug, Clone)]
struct Directive {
    name: Option<String>,
    level: LevelFilter,
}

impl Directive {
    fn new<T: Into<String>>(name: Option<T>, level: LevelFilter) -> Self {
        Self {
            name: name.map(Into::into),
            level,
        }
    }
}

impl FromStr for Directive {
    type Err = FilterParseError;

    /// convert string such as follows to Directive
    ///
    /// * crate1::mod1=error : module name and level
    /// * crate2= : only a module name ends with equal
    /// * info : only a level
    /// * crate3::foo : only a module name
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('=').map(str::trim);
        let (name, level) = match (parts.next(), parts.next(), parts.next()) {
            (Some(level_or_module), None, None) => match level_or_module.parse() {
                Ok(level) => (None, level),
                Err(_) => (Some(level_or_module), LevelFilter::max()),
            },
            (Some(name), Some(""), None) => (Some(name), LevelFilter::max()),
            (Some(name), Some(level), None) => (Some(name), level.parse()?),
            _ => return Err(FilterParseError),
        };

        Ok(Directive::new(name, level))
    }
}
