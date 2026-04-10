use super::{TimeStyle, TracingConfig};

use std::fmt;

use tracing_subscriber::fmt::{
    format::Writer,
    time::{ChronoLocal, ChronoUtc, FormatTime, SystemTime, Uptime},
};

const DEFAULT_LOCAL_TIME_PATTERN: &str = "%Y-%m-%d %H:%M:%S";
const DEFAULT_UTC_TIME_PATTERN: &str = "%Y-%m-%dT%H:%M:%SZ";

#[derive(Clone, Debug)]
pub enum TimestampFormatter {
    None,
    System(SystemTime),
    Uptime(Uptime),
    Local(ChronoLocal),
    Utc(ChronoUtc),
}

impl TimestampFormatter {
    pub fn from_config(config: &TracingConfig) -> Self {
        match config.time_style {
            TimeStyle::None => Self::None,
            TimeStyle::System => Self::System(SystemTime),
            TimeStyle::Uptime => Self::Uptime(Uptime::default()),
            TimeStyle::Local => Self::Local(ChronoLocal::new(
                config
                    .time_pattern
                    .clone()
                    .unwrap_or_else(|| DEFAULT_LOCAL_TIME_PATTERN.to_string()),
            )),
            TimeStyle::Utc => Self::Utc(ChronoUtc::new(
                config
                    .time_pattern
                    .clone()
                    .unwrap_or_else(|| DEFAULT_UTC_TIME_PATTERN.to_string()),
            )),
        }
    }
}

impl FormatTime for TimestampFormatter {
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::System(timer) => timer.format_time(w),
            Self::Uptime(timer) => timer.format_time(w),
            Self::Local(timer) => timer.format_time(w),
            Self::Utc(timer) => timer.format_time(w),
        }
    }
}
