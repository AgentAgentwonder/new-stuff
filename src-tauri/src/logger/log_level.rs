use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
    Success = 6,
    Performance = 7,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
            LogLevel::Success => "SUCCESS",
            LogLevel::Performance => "PERFORMANCE",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TRACE" => Some(LogLevel::Trace),
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARN" => Some(LogLevel::Warn),
            "ERROR" => Some(LogLevel::Error),
            "FATAL" => Some(LogLevel::Fatal),
            "SUCCESS" => Some(LogLevel::Success),
            "PERFORMANCE" => Some(LogLevel::Performance),
            _ => None,
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            LogLevel::Trace => "\x1b[90m",       // Gray
            LogLevel::Debug => "\x1b[36m",       // Cyan
            LogLevel::Info => "\x1b[34m",        // Blue
            LogLevel::Warn => "\x1b[33m",        // Yellow
            LogLevel::Error => "\x1b[31m",       // Red
            LogLevel::Fatal => "\x1b[35;1m",     // Bright Magenta
            LogLevel::Success => "\x1b[32m",     // Green
            LogLevel::Performance => "\x1b[95m", // Light Magenta
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
