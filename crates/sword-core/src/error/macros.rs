#[macro_export]
macro_rules! sword_error {
    (
        title: $title:expr,
        reason: $reason:expr,
        $(source: $source:expr,)?
        fields: { $($field:ident = $value:expr),* $(,)? },
        hints: [$hint:expr $(,)?],
        fatal: false
        $(,)?
    ) => {{
        let title = ($title).to_string();
        let reason = ($reason).to_string();

        if tracing::dispatcher::has_been_set() {
            tracing::error!(
                target: "sword.startup.error",
                $(source = %$source,)?
                reason = %reason,
                "{}",
                title
            );

            tracing::debug!(
                target: "sword.startup.error",
                $(source = %$source,)?
                $($field = %$value,)*
                hint = %$hint,
                "Startup diagnostic details"
            );
        } else {
            eprintln!("ERROR: {title}");
            eprintln!("Enable tracing in config to see details");
        }
    }};

    (
        title: $title:expr,
        reason: $reason:expr,
        $(source: $source:expr,)?
        fields: { $($field:ident = $value:expr),* $(,)? },
        hints: [$hint:expr $(,)?]
        $(, fatal: true)?
        $(,)?
    ) => {{
        let title = ($title).to_string();
        let reason = ($reason).to_string();

        if tracing::dispatcher::has_been_set() {
            tracing::error!(
                target: "sword.startup.error",
                $(source = %$source,)?
                reason = %reason,
                "{}",
                title
            );

            tracing::debug!(
                target: "sword.startup.error",
                $(source = %$source,)?
                $($field = %$value,)*
                hint = %$hint,
                "Startup diagnostic details"
            );
        } else {
            eprintln!("ERROR: {title}");
            eprintln!("Enable tracing in config to see details");
        }

        panic!("fatal sword diagnostic emitted")
    }};

    (
        title: $title:expr,
        reason: $reason:expr
        $(, source: $source:expr)?
        $(, context: { $($context_key:expr => $context_value:expr),* $(,)? })?
        $(, extra_context: $extra_context:expr)?
        $(, hints: [ $($hint:expr),* $(,)? ])?
        , fatal: false
        $(,)?
    ) => {{
        let mut diagnostic =
            $crate::error::StartupDiagnostic::new(($title).to_string(), ($reason).to_string());

        $(
            diagnostic = diagnostic.with_source(($source).to_string());
        )?

        $(
            $(
                diagnostic = diagnostic.add_context(
                    ($context_key).to_string(),
                    ($context_value).to_string(),
                );
            )*
        )?

        $(
            diagnostic = diagnostic.extend_context(($extra_context).into_iter());
        )?

        $(
            $(
                diagnostic = diagnostic.add_hint(($hint).to_string());
            )*
        )?

        $crate::error::emit(diagnostic);
    }};

    (
        title: $title:expr,
        reason: $reason:expr
        $(, source: $source:expr)?
        $(, context: { $($context_key:expr => $context_value:expr),* $(,)? })?
        $(, extra_context: $extra_context:expr)?
        $(, hints: [ $($hint:expr),* $(,)? ])?
        $(, fatal: true)?
        $(,)?
    ) => {{
        let mut diagnostic =
            $crate::error::StartupDiagnostic::new(($title).to_string(), ($reason).to_string());

        $(
            diagnostic = diagnostic.with_source(($source).to_string());
        )?

        $(
            $(
                diagnostic = diagnostic.add_context(
                    ($context_key).to_string(),
                    ($context_value).to_string(),
                );
            )*
        )?

        $(
            diagnostic = diagnostic.extend_context(($extra_context).into_iter());
        )?

        $(
            $(
                diagnostic = diagnostic.add_hint(($hint).to_string());
            )*
        )?

        $crate::error::emit(diagnostic);
        panic!("fatal sword diagnostic emitted")
    }};
}
