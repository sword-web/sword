use std::fmt;
use tracing::{
    Event, Level, Subscriber,
    field::{Field, Visit},
};

use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer, time::FormatTime},
    registry::LookupSpan,
};

use super::{TracingConfig, TracingField, time::TimestampFormatter};

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_KEY: &str = "\x1b[36m";
const ANSI_ERROR: &str = "\x1b[31m";
const ANSI_WARN: &str = "\x1b[33m";
const ANSI_INFO: &str = "\x1b[32m";
const ANSI_DEBUG: &str = "\x1b[34m";
const ANSI_TRACE: &str = "\x1b[90m";

/// Field name used by `tracing` for the human-readable event message.
const MESSAGE_FIELD: &str = "message";

/// Indentation for a field rendered below the event message.
const FIELD_INDENT: &str = "\n       ";

/// Indentation for the lines inside a multi-line field block.
const BLOCK_LINE_INDENT: &str = "         ";

#[derive(Clone, Debug)]
pub struct DevFormatter {
    with_target: bool,
    with_file: bool,
    with_line_number: bool,
    timer: TimestampFormatter,
}

impl DevFormatter {
    pub fn new(config: &TracingConfig) -> Self {
        Self {
            with_target: config.has_field(TracingField::Target),
            with_file: config.has_field(TracingField::File),
            with_line_number: config.has_field(TracingField::LineNumber),
            timer: TimestampFormatter::from_config(config),
        }
    }

    fn write_key(&self, writer: &mut Writer<'_>, key: &str) -> fmt::Result {
        if writer.has_ansi_escapes() {
            write!(writer, "{ANSI_KEY}{key}{ANSI_RESET}")
        } else {
            writer.write_str(key)
        }
    }

    fn write_level(&self, writer: &mut Writer<'_>, level: &Level) -> fmt::Result {
        if writer.has_ansi_escapes() {
            let color = match *level {
                Level::ERROR => ANSI_ERROR,
                Level::WARN => ANSI_WARN,
                Level::INFO => ANSI_INFO,
                Level::DEBUG => ANSI_DEBUG,
                Level::TRACE => ANSI_TRACE,
            };

            write!(writer, "{color}{level:>5}{ANSI_RESET}  ")
        } else {
            write!(writer, "{level:>5}  ")
        }
    }

    fn write_field(&self, writer: &mut Writer<'_>, key: &str, value: &str) -> fmt::Result {
        writer.write_str("  ")?;
        self.write_key(writer, key)?;

        writer.write_str(": ")?;
        writer.write_str(value)
    }

    fn write_multiline_field(
        &self,
        writer: &mut Writer<'_>,
        key: &str,
        value: &str,
    ) -> fmt::Result {
        writer.write_str(FIELD_INDENT)?;
        self.write_key(writer, key)?;
        writer.write_str(":")?;

        for line in value.trim_end().lines() {
            writer.write_str("\n")?;

            if !line.is_empty() {
                writer.write_str(BLOCK_LINE_INDENT)?;
                writer.write_str(line)?;
            }
        }

        Ok(())
    }
}

impl<S, N> FormatEvent<S, N> for DevFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        _: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let metadata = event.metadata();
        let mut visitor = DevFieldVisitor::default();

        event.record(&mut visitor);

        self.timer.format_time(&mut writer)?;

        if !matches!(self.timer, TimestampFormatter::None) {
            writer.write_str(" ")?;
        }

        self.write_level(&mut writer, metadata.level())?;

        if let Some(message) = visitor.message.as_deref() {
            writer.write_str("  ")?;
            writer.write_str(message)?;
        }

        if self.with_target {
            self.write_field(&mut writer, "target", metadata.target())?;
        }

        if self.with_file
            && let Some(file) = metadata.file()
        {
            self.write_field(&mut writer, "file", file)?;
        }

        if self.with_line_number
            && let Some(line_number) = metadata.line()
        {
            let line_number = line_number.to_string();
            self.write_field(&mut writer, "line", &line_number)?;
        }

        for (key, value) in &visitor.fields {
            if value.contains('\n') {
                self.write_multiline_field(&mut writer, key, value)?;
            } else {
                self.write_field(&mut writer, key, value)?;
            }
        }

        writer.write_str("\n")
    }
}

#[derive(Default)]
struct DevFieldVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl DevFieldVisitor {
    fn push(&mut self, field: &Field, value: String) {
        if field.name() == MESSAGE_FIELD {
            self.message = Some(value);
        } else {
            self.fields.push((field.name().to_string(), value));
        }
    }
}

impl Visit for DevFieldVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.push(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.push(field, value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.push(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.push(field, value.to_string());
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.push(field, value.to_string());
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.push(field, value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.push(field, value.to_string());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.push(field, value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.push(field, format!("{value:?}"));
    }
}
