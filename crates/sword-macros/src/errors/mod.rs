mod grpc_error;
#[cfg(feature = "web-controllers")]
mod http_error;
mod shared;

pub use grpc_error::*;
#[cfg(feature = "web-controllers")]
pub use http_error::*;
pub use shared::*;

use syn::{Ident, LitStr, Token, meta::ParseNestedMeta};

#[derive(Debug, Clone)]
pub enum MessageValue {
    Static(String),
    Field(String),
    Interpolated(String),
}

impl MessageValue {
    pub fn parse(ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<MessageValue> {
        if !meta.input.peek(Token![=]) {
            return Err(syn::Error::new(
                ident.span(),
                "expected '=' after 'message'",
            ));
        }

        meta.input.parse::<Token![=]>()?;

        if let Ok(lit) = meta.input.parse::<LitStr>() {
            let value = lit.value();

            if value.contains('{') || value.contains('}') {
                return Ok(MessageValue::Interpolated(value));
            }

            return Ok(MessageValue::Static(value));
        }

        if let Ok(field) = meta.input.parse::<Ident>() {
            return Ok(MessageValue::Field(field.to_string()));
        }

        Err(syn::Error::new(
            ident.span(),
            "expected string literal or field identifier",
        ))
    }
}

/// Extracts field names from `{field_name}` patterns in an interpolated message template.
/// Recognizes `{{` and `}}` as format ! escaping and skips them.
pub fn extract_template_fields(template: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut chars = template.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
                continue;
            }
            let mut name = String::new();
            while let Some(&c) = chars.peek() {
                if c == '}' {
                    break;
                }
                name.push(c);
                chars.next();
            }
            if !name.is_empty() {
                fields.push(name);
            }
            chars.next();
        }
    }
    fields
}

/// Converts an interpolated template into a `format!` format string and a list of field names.
/// `"Conflict on {field}: {value}"` → `("Conflict on {}: {}", ["field", "value"])`
pub fn format_template(template: &str) -> (String, Vec<String>) {
    let mut fields = Vec::new();
    let mut format_str = String::new();
    let mut chars = template.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
                format_str.push('{');
                continue;
            }
            let mut name = String::new();
            while let Some(&c) = chars.peek() {
                if c == '}' {
                    break;
                }
                name.push(c);
                chars.next();
            }
            if !name.is_empty() {
                fields.push(name);
                format_str.push_str("{}");
            }
            chars.next();
        } else {
            format_str.push(ch);
        }
    }
    (format_str, fields)
}
