//! Scalar API reference page, served instead of Swagger UI when the `scalar-ui` feature is on.
//!
//! The page is a small self-contained HTML document that loads the Scalar bundle from a CDN
//! and points it at the OpenAPI specs mounted by the router.

use axum::response::Html;
use serde_json::json;

/// Scalar bundle loaded by the generated page. Pinned to a major version so a breaking release
/// can not change the UI under existing applications.
const SCALAR_CDN_URL: &str = "https://cdn.jsdelivr.net/npm/@scalar/api-reference@1";

/// Renders the Scalar HTML page for the given spec URLs.
///
/// The first spec is the default one; additional specs show up in Scalar's source selector.
pub(super) fn render_html(title: &str, spec_urls: &[String]) -> Html<String> {
    let sources: Vec<_> = spec_urls
        .iter()
        .map(|url| {
            let name = url.rsplit('/').next().unwrap_or(url);
            json!({ "url": url, "title": name })
        })
        .collect();

    // `<` is escaped so no value can close the inline script block.
    let configuration = json!({ "sources": sources })
        .to_string()
        .replace('<', "\\u003c");

    Html(format!(
        r##"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{title}</title>
  </head>
  <body>
    <div id="app"></div>
    <script src="{SCALAR_CDN_URL}"></script>
    <script>
      Scalar.createApiReference("#app", {configuration});
    </script>
  </body>
</html>
"##,
        title = escape_html(title),
    ))
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_every_spec_as_a_source() {
        let urls = vec![
            "/api/openapi/a.yaml".to_string(),
            "/api/openapi/b.json".to_string(),
        ];
        let html = render_html("Docs", &urls).0;

        assert!(html.contains(r#""url":"/api/openapi/a.yaml""#));
        assert!(html.contains(r#""title":"b.json""#));
    }

    #[test]
    fn escapes_title_and_script_content() {
        let urls = vec!["/x</script><b>.json".to_string()];
        let html = render_html("<Docs & \"co\">", &urls).0;

        assert!(html.contains("&lt;Docs &amp; &quot;co&quot;&gt;"));
        assert!(!html.contains("</script><b>"));
    }
}
