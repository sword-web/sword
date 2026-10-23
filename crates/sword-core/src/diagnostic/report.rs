use super::Diagnostic;
use super::renderer::{self, Renderer};
use super::types::{DiagnosticPhase, Severity};

impl Diagnostic {
    /// Reports this diagnostic with the renderer selected for the process.
    ///
    /// `[application] diagnostics` chooses between miette and tracing. When the
    /// configured renderer is tracing but no subscriber is installed, we fall
    /// back to miette instead of dropping the diagnostic.
    pub fn report(&self) {
        match renderer::current() {
            Renderer::Miette => self.report_miette(),
            Renderer::Tracing if tracing::dispatcher::has_been_set() => self.report_tracing(),
            Renderer::Tracing => self.report_miette(),
        }
    }

    fn report_miette(&self) {
        let report = miette::Report::new(self.clone());
        let rendered = format!("{report:?}");

        eprint!("{}", Self::space_sections(&rendered));
    }

    /// miette packs the message, the snippet and the help block together.
    /// Adding a blank line before each block makes the report easier to scan.
    fn space_sections(rendered: &str) -> String {
        let mut output = String::with_capacity(rendered.len() + 2);
        let mut snippet_separated = false;
        let mut help_separated = false;

        for line in rendered.split_inclusive('\n') {
            if !snippet_separated && Self::is_snippet_start(line) {
                Self::ensure_blank_line(&mut output);
                snippet_separated = true;
            }

            if !help_separated && Self::is_help_start(line) {
                Self::ensure_blank_line(&mut output);
                help_separated = true;
            }

            output.push_str(line);
        }

        if !output.ends_with('\n') {
            output.push('\n');
        }

        output
    }

    fn ensure_blank_line(output: &mut String) {
        if output.is_empty() || output.ends_with("\n\n") {
            return;
        }

        output.push('\n');
    }

    fn is_snippet_start(line: &str) -> bool {
        ["╭─[", "╭─▶", "+--[", "+--▶", "┌─["]
            .iter()
            .any(|prefix| line.contains(prefix))
    }

    fn is_help_start(line: &str) -> bool {
        line.contains("help:")
    }

    fn report_tracing(&self) {
        let code = self.code().as_str();
        let phase = self.phase().map(DiagnosticPhase::label).unwrap_or("none");
        let engine = self.engine().unwrap_or("none");
        let source = self.source().unwrap_or("none");
        let reason = self.reason();
        let title = self.title();

        match self.severity() {
            Severity::Error => tracing::error!(
                target: "sword.diagnostic",
                code,
                phase,
                engine,
                source,
                reason = %reason,
                "{}",
                title
            ),
            Severity::Warning => tracing::warn!(
                target: "sword.diagnostic",
                code,
                phase,
                engine,
                source,
                reason = %reason,
                "{}",
                title
            ),
            Severity::Advice => tracing::info!(
                target: "sword.diagnostic",
                code,
                phase,
                engine,
                source,
                reason = %reason,
                "{}",
                title
            ),
        }
    }
}
