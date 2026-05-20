use axum::{
    body::Body,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
};

/// A builder for creating file download/inline responses.
///
/// # Example
///
/// ```rust
/// use axum_responses::{File, ContentDisposition};
///
/// async fn download() -> File {
///     let data = std::fs::read("report.pdf").unwrap();
///     File::new()
///         .bytes(&data)
///         .content_type("application/pdf")
///         .filename("report.pdf")
///         .attachment()
/// }
/// ```
#[derive(Debug)]
pub struct File {
    bytes: Vec<u8>,
    content_type: &'static str,
    filename: Option<&'static str>,
    disposition: ContentDisposition,
    headers: HeaderMap,
}

/// Specifies how the content should be presented to the user.
#[derive(Debug, Default, Clone, Copy)]
pub enum ContentDisposition {
    /// The content is displayed directly in the browser.
    Inline,

    /// The content is treated as a downloadable file.
    #[default]
    Attachment,
}

impl Default for File {
    fn default() -> Self {
        Self::new()
    }
}

impl File {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            content_type: "application/octet-stream",
            filename: None,
            disposition: ContentDisposition::Attachment,
            headers: HeaderMap::new(),
        }
    }

    /// Sets the file content as a byte slice.
    pub fn bytes(mut self, bytes: &[u8]) -> Self {
        self.bytes = bytes.to_vec();
        self
    }

    /// Sets the content type of the file.
    pub fn content_type(mut self, content_type: &'static str) -> Self {
        self.content_type = content_type;
        self
    }

    /// Sets the filename for the Content-Disposition header.
    pub fn filename(mut self, filename: &'static str) -> Self {
        self.filename = Some(filename);
        self
    }

    /// Sets the content disposition (inline or attachment).
    pub fn disposition(mut self, disposition: ContentDisposition) -> Self {
        self.disposition = disposition;
        self
    }

    /// Shorthand for `disposition(ContentDisposition::Attachment)`.
    pub fn attachment(mut self) -> Self {
        self.disposition = ContentDisposition::Attachment;
        self
    }

    /// Shorthand for `disposition(ContentDisposition::Inline)`.
    pub fn inline(mut self) -> Self {
        self.disposition = ContentDisposition::Inline;
        self
    }

    /// Adds a custom header to the response.
    pub fn header(mut self, key: &'static str, value: &'static str) -> Self {
        if let (Ok(header_name), Ok(header_value)) =
            (HeaderName::try_from(key), HeaderValue::try_from(value))
        {
            self.headers.insert(header_name, header_value);
        }
        self
    }
}

impl IntoResponse for File {
    fn into_response(self) -> AxumResponse {
        let disposition = match self.disposition {
            ContentDisposition::Inline => "inline",
            ContentDisposition::Attachment => "attachment",
        };

        let filename = self.filename.unwrap_or("file");
        let content_disposition = format!("{disposition}; filename=\"{filename}\"");

        let headers = [
            ("Content-Type", self.content_type),
            ("Content-Disposition", &content_disposition),
        ];

        (StatusCode::OK, headers, Body::from(self.bytes)).into_response()
    }
}
