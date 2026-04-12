use serde::{Deserialize, Serialize};
use thisconfig::ConfigItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TracingConfig {
    /// Enables or disables global tracing initialization.
    ///
    /// When `false`, `TracingSubscriber::init` returns without registering a
    /// subscriber.
    pub enabled: bool,
    /// Reads filter directives from the `RUST_LOG` environment variable.
    ///
    /// If enabled, the environment value is attempted first. If parsing fails
    /// or the variable is missing, `default_filter` is used as fallback.
    #[serde(rename = "use-env-filter")]
    pub use_env_filter: bool,
    /// Fallback filter directives for `tracing_subscriber::EnvFilter`.
    ///
    /// Accepts standard tracing directives like `info`, `warn,my_crate=debug`,
    /// etc. This value is always used when `use_env_filter` is `false`.
    #[serde(rename = "filter")]
    pub filter: String,
    /// Event formatting style used by the subscriber output.
    ///
    /// `full` follows tracing-subscriber defaults, while `pretty`, `compact`,
    /// `dev` and `json` provide alternative renderers.
    pub format: LogFormat,
    /// Optional metadata fields to include in each event record.
    ///
    /// Supported values are `target`, `file`, `line-number`, `thread-id`, and
    /// `thread-name`.
    #[serde(rename = "with-fields")]
    pub with_fields: Vec<TracingField>,
    /// Selects which clock source is used for timestamps.
    ///
    /// `system` uses tracing-subscriber's default wall-clock formatter,
    /// `uptime` shows elapsed time since subscriber initialization, `local`
    /// and `utc` render formatted calendar timestamps, and `none` suppresses
    /// timestamps even when `timer` is `true`.
    #[serde(rename = "time-style")]
    pub time_style: TimeStyle,
    /// Custom strftime pattern used by `local` and `utc` timestamp styles.
    ///
    /// This is ignored by `system`, `uptime`, and `none`.
    #[serde(rename = "time-pattern")]
    pub time_pattern: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Full,
    Pretty,
    Compact,
    /// Single-line developer-friendly console output.
    ///
    /// This format keeps logs compact without falling back to `key=value`
    /// rendering for every field. Event messages are printed first and the
    /// remaining fields are rendered as `key: value`, optionally coloring only
    /// the keys when ANSI is enabled.
    Dev,
    Json,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TracingField {
    Target,
    File,
    LineNumber,
    ThreadId,
    ThreadName,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TimeStyle {
    #[default]
    System,
    Uptime,
    Local,
    Utc,
    None,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            use_env_filter: true,
            filter: "info".to_string(),
            format: LogFormat::Full,
            with_fields: vec![TracingField::Target],
            time_style: TimeStyle::System,
            time_pattern: None,
        }
    }
}

impl ConfigItem for TracingConfig {
    fn key() -> &'static str {
        "tracing"
    }
}

impl TracingConfig {
    pub fn has_field(&self, field: TracingField) -> bool {
        self.with_fields.contains(&field)
    }
}
