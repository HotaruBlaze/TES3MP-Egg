use ansiterm::Color;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Unknown,
}

#[derive(Clone)]
pub struct LogColorizer {
    error_regex: Regex,
    warn_regex: Regex,
    info_regex: Regex,
    debug_regex: Regex,
}

impl LogColorizer {
    pub fn new() -> Self {
        Self {
            error_regex: Regex::new(r"(?i)\[?ERROR\]?|\[?FATAL\]?").unwrap(),
            warn_regex: Regex::new(r"(?i)\[?WARNING\]?|\[?WARN\]?").unwrap(),
            info_regex: Regex::new(r"(?i)\[?INFO\]?").unwrap(),
            debug_regex: Regex::new(r"(?i)\[?DEBUG\]?").unwrap(),
        }
    }

    pub fn detect_level(&self, line: &str) -> LogLevel {
        if self.error_regex.is_match(line) {
            LogLevel::Error
        } else if self.warn_regex.is_match(line) {
            LogLevel::Warn
        } else if self.info_regex.is_match(line) {
            LogLevel::Info
        } else if self.debug_regex.is_match(line) {
            LogLevel::Debug
        } else {
            LogLevel::Unknown
        }
    }

    pub fn colorize(&self, line: &str) -> String {
        match self.detect_level(line) {
            LogLevel::Error => format!(
                "{}{}{}",
                Color::Red.prefix(),
                line,
                Color::Red.suffix()
            ),
            LogLevel::Warn => format!(
                "{}{}{}",
                Color::Yellow.prefix(),
                line,
                Color::Yellow.suffix()
            ),
            LogLevel::Info => format!(
                "{}{}{}",
                Color::Green.prefix(),
                line,
                Color::Green.suffix()
            ),
            LogLevel::Debug => format!(
                "{}{}{}",
                Color::Blue.prefix(),
                line,
                Color::Blue.suffix()
            ),
            LogLevel::Unknown => line.to_string(),
        }
    }
}

impl Default for LogColorizer {
    fn default() -> Self {
        Self::new()
    }
}
