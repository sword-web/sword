#[derive(Debug, Clone)]
pub struct StartupDiagnostic {
    title: String,
    reason: String,
    source: Option<String>,
    context: Vec<(String, String)>,
    hints: Vec<String>,
}

impl StartupDiagnostic {
    pub fn new(title: String, reason: String) -> Self {
        Self {
            title,
            reason,
            source: None,
            context: Vec::new(),
            hints: Vec::new(),
        }
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    pub fn add_context(mut self, key: String, value: String) -> Self {
        self.context.push((key, value));
        self
    }

    pub fn extend_context<I, K, V>(mut self, context: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.context.extend(
            context
                .into_iter()
                .map(|(key, value)| (key.into(), value.into())),
        );

        self
    }

    pub fn add_hint(mut self, hint: String) -> Self {
        self.hints.push(hint);
        self
    }

    fn has_details(&self) -> bool {
        !self.context.is_empty() || !self.hints.is_empty()
    }
}

pub fn emit(diagnostic: StartupDiagnostic) {
    if tracing::dispatcher::has_been_set() {
        emit_tracing_logs(&diagnostic);
    } else {
        eprintln!("ERROR: {}", diagnostic.title);
        eprintln!("Reason: {}", diagnostic.reason);
        eprintln!(
            "Source: {}",
            diagnostic.source.as_deref().unwrap_or("Unknown")
        );
        eprintln!("Context:");
        for (key, value) in &diagnostic.context {
            eprintln!("  {}: {}", key, value);
        }
        eprintln!("Hints:");
        for hint in &diagnostic.hints {
            eprintln!("  {}", hint);
        }
        eprintln!("Enable tracing in config to see details");
    }
}

fn emit_tracing_logs(diagnostic: &StartupDiagnostic) {
    if let Some(source) = diagnostic.source.as_deref() {
        tracing::error!(
            target: "sword.startup.error",
            source,
            reason = %diagnostic.reason,
            "{}",
            diagnostic.title
        );

        if diagnostic.has_details() {
            emit_context_summary_error(Some(source), &diagnostic.context);

            for (key, value) in &diagnostic.context {
                tracing::debug!(
                    target: "sword.startup.error",
                    source,
                    context_key = %key,
                    context_value = %value,
                    "Startup diagnostic context"
                );
            }

            for hint in &diagnostic.hints {
                tracing::debug!(
                    target: "sword.startup.error",
                    source,
                    hint = %hint,
                    "Startup diagnostic hint"
                );
            }
        }

        return;
    }

    tracing::error!(
        target: "sword.startup.error",
        reason = %diagnostic.reason,
        "{}",
        diagnostic.title
    );

    if diagnostic.has_details() {
        emit_context_summary_error(None, &diagnostic.context);

        for (key, value) in &diagnostic.context {
            tracing::debug!(
                target: "sword.startup.error",
                context_key = %key,
                context_value = %value,
                "Startup diagnostic context"
            );
        }

        for hint in &diagnostic.hints {
            tracing::debug!(
                target: "sword.startup.error",
                hint = %hint,
                "Startup diagnostic hint"
            );
        }
    }
}

fn emit_context_summary_error(source: Option<&str>, context: &[(String, String)]) {
    if context.is_empty() {
        return;
    }

    const PRIORITY_KEYS: [&str; 6] = [
        "missing_dependency_path",
        "dependency_path",
        "controller_name",
        "handler_id",
        "path",
        "bind",
    ];

    let mut selected: Vec<(&str, &str)> = Vec::new();

    for key in PRIORITY_KEYS {
        if let Some((found_key, found_value)) = context
            .iter()
            .find(|(ctx_key, _)| ctx_key.as_str() == key)
            .map(|(ctx_key, ctx_value)| (ctx_key.as_str(), ctx_value.as_str()))
        {
            selected.push((found_key, found_value));
        }
    }

    if selected.is_empty() {
        selected.extend(
            context
                .iter()
                .take(2)
                .map(|(key, value)| (key.as_str(), value.as_str())),
        );
    }

    for (key, value) in selected {
        if let Some(source) = source {
            tracing::error!(
                target: "sword.startup.error",
                source,
                context_key = %key,
                context_value = %value,
                "Startup diagnostic key context"
            );
        } else {
            tracing::error!(
                target: "sword.startup.error",
                context_key = %key,
                context_value = %value,
                "Startup diagnostic key context"
            );
        }
    }
}
