/// Builds an [`Error`](crate::diagnostic::Error) from a diagnostic description.
///
/// This macro only constructs the error. It does not print it, does not report
/// it and does not panic, so it can be used anywhere a `Result` is expected:
///
/// ```ignore
/// return Err(sword_error! {
///     code: "sword.build.config",
///     phase: DiagnosticPhase::Build,
///     title: "Failed to load configuration",
///     reason: err,
///     source: "ApplicationBuilder::from_config",
///     context: { "path" => config_path },
///     hints: ["Ensure the file exists and contains valid TOML"],
///     cause: err,
/// });
/// ```
///
/// The fields keep a fixed order so the required ones (`code`, `title` and
/// `reason`) are always present. Optional fields can be omitted, but they cannot
/// be reordered.
#[macro_export]
macro_rules! sword_error {
    (
        code: $code:expr
        $(, phase: $phase:expr)?
        $(, engine: $engine:expr)?
        $(, target: $target:expr)?
        , title: $title:expr
        , reason: $reason:expr
        $(, source: $source:expr)?
        $(, source_code: { name: $source_name:expr, text: $source_text:expr, })?
        $(, label: { span: $label_span:expr, message: $label_message:expr, })?
        $(, source_location: { file: $location_file:expr, line: $location_line:expr, column: $location_column:expr, length: $location_len:expr, message: $location_message:expr, })?
        $(, context: { $($key:expr => $value:expr),* $(,)? })?
        $(, extra_context: $extra_context:expr)?
        $(, hints: [ $($hint:expr),* $(,)? ])?
        $(, cause: $cause:expr)?
        $(,)?
    ) => {{
        let mut builder = $crate::diagnostic::Diagnostic::builder()
            .code($code)
            .title(($title).to_string())
            .reason(($reason).to_string());

        $(
            builder = builder.phase($phase);
        )?

        $(
            builder = builder.engine($engine);
        )?

        $(
            builder = builder.target(($target).to_string());
        )?

        $(
            builder = builder.source(($source).to_string());
        )?

        $(
            builder = builder.source_code(
                ($source_name).to_string(),
                ($source_text).to_string(),
            );
        )?

        $(
            builder = builder.label($label_span, ($label_message).to_string());
        )?

        $(
            builder = builder.source_location(
                ($location_file).to_string(),
                $location_line as usize,
                $location_column as usize,
                $location_len as usize,
                ($location_message).to_string(),
            );
        )?

        $(
            $(
                builder = builder.context(
                    ($key).to_string(),
                    ($value).to_string(),
                );
            )*
        )?

        $(
            for (key, value) in ($extra_context).into_iter() {
                builder = builder.context(key, value);
            }
        )?

        $(
            $(
                builder = builder.hint(($hint).to_string());
            )*
        )?

        let error = $crate::diagnostic::Error::new(builder.build());

        $(
            let error = error.with_source($cause);
        )?

        error
    }};
}
