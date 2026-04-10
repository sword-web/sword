use std::fmt;
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

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

#[derive(Clone, Debug)]
pub struct DevFormatter {
    with_target: bool,
    with_file: bool,
    with_line_number: bool,
    timer: TimestampFormatter,
    last_event_layout: Arc<AtomicU8>,
}

impl DevFormatter {
    pub fn new(config: &TracingConfig) -> Self {
        Self {
            with_target: config.has_field(TracingField::Target),
            with_file: config.has_field(TracingField::File),
            with_line_number: config.has_field(TracingField::LineNumber),
            timer: TimestampFormatter::from_config(config),
            last_event_layout: Arc::new(AtomicU8::new(0)),
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

            write!(writer, "{color}{level}{ANSI_RESET}")
        } else {
            write!(writer, "{level}")
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
        writer.write_str("\n       ")?;
        self.write_key(writer, key)?;
        writer.write_str(":")?;

        let lines: Vec<&str> = value.lines().collect();

        if lines.is_empty() {
            return Ok(());
        }

        let render_as_block = lines.len() > 1 || lines[0].starts_with("- ");

        if !render_as_block {
            writer.write_str(" ")?;
            writer.write_str(lines[0])?;
            return Ok(());
        }

        for line in lines {
            writer.write_str("\n         ")?;
            writer.write_str(line)?;
        }

        Ok(())
    }

    fn should_use_multiline(&self, level: &Level, visitor: &DevFieldVisitor) -> bool {
        const MAX_SINGLE_LINE_MESSAGE: usize = 100;
        const MAX_SINGLE_LINE_FIELD: usize = 60;
        const MAX_SINGLE_LINE_FIELDS: usize = 3;
        const MAX_SINGLE_LINE_TOTAL: usize = 140;
        const MAX_SINGLE_LINE_TOTAL_ERROR: usize = 120;

        let summary = EventLayoutSummary::from_visitor(visitor);

        if summary.message_has_newline || summary.field_has_newline {
            return true;
        }

        if summary.is_diagnostic {
            return true;
        }

        if summary.message_len > MAX_SINGLE_LINE_MESSAGE {
            return true;
        }

        if summary.max_field_len > MAX_SINGLE_LINE_FIELD {
            return true;
        }

        if summary.field_count > MAX_SINGLE_LINE_FIELDS {
            return true;
        }

        if *level == Level::ERROR && summary.total_len > MAX_SINGLE_LINE_TOTAL_ERROR {
            return true;
        }

        summary.total_len > MAX_SINGLE_LINE_TOTAL
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
        let previous_layout = self.last_event_layout.load(Ordering::Relaxed);

        let metadata = event.metadata();
        let mut visitor = DevFieldVisitor::default();
        event.record(&mut visitor);

        let multiline = self.should_use_multiline(metadata.level(), &visitor);

        if previous_layout != 0 && (previous_layout == 2 || multiline) {
            writer.write_str("\n")?;
        }

        self.timer.format_time(&mut writer)?;

        if !matches!(self.timer, TimestampFormatter::None) {
            writer.write_str(" ")?;
        }

        self.write_level(&mut writer, metadata.level())?;

        if let Some(message) = visitor.message.as_deref() {
            writer.write_str("  ")?;
            writer.write_str(message)?;
        }

        if multiline {
            if self.with_target {
                self.write_multiline_field(&mut writer, "target", metadata.target())?;
            }

            if self.with_file
                && let Some(file) = metadata.file()
            {
                self.write_multiline_field(&mut writer, "file", file)?;
            }

            if self.with_line_number
                && let Some(line_number) = metadata.line()
            {
                let line_number = line_number.to_string();
                self.write_multiline_field(&mut writer, "line", &line_number)?;
            }

            for (key, value) in &visitor.fields {
                self.write_multiline_field(&mut writer, key, value)?;
            }

            writer.write_str("\n")?;
            self.last_event_layout.store(2, Ordering::Relaxed);
            return Ok(());
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
            self.write_field(&mut writer, key, value)?;
        }

        self.last_event_layout.store(1, Ordering::Relaxed);
        writer.write_str("\n")
    }
}

struct EventLayoutSummary {
    field_count: usize,
    message_len: usize,
    max_field_len: usize,
    total_len: usize,
    message_has_newline: bool,
    field_has_newline: bool,
    is_diagnostic: bool,
}

impl EventLayoutSummary {
    fn from_visitor(visitor: &DevFieldVisitor) -> Self {
        let message = visitor.message.as_deref().unwrap_or_default();
        let field_count = visitor.fields.len();
        let message_len = message.len();
        let message_has_newline = message.contains('\n');
        let field_has_newline = visitor.fields.iter().any(|(_, value)| value.contains('\n'));
        let max_field_len = visitor
            .fields
            .iter()
            .map(|(_, value)| value.len())
            .max()
            .unwrap_or(0);
        let total_len = message_len
            + visitor
                .fields
                .iter()
                .map(|(key, value)| key.len() + value.len() + 4)
                .sum::<usize>();

        let is_diagnostic = visitor.fields.iter().any(|(key, _)| {
            matches!(
                key.as_str(),
                "reason"
                    | "source"
                    | "hint"
                    | "hints"
                    | "details"
                    | "dependency_path"
                    | "missing_dependency_path"
            )
        });

        Self {
            field_count,
            message_len,
            max_field_len,
            total_len,
            message_has_newline,
            field_has_newline,
            is_diagnostic,
        }
    }
}

#[derive(Default)]
struct DevFieldVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl DevFieldVisitor {
    fn push(&mut self, field: &Field, value: String) {
        if field.name() == "message" {
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
