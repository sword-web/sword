//! Security headers middleware.
//!
//! This module wraps `axum-helmet` to provide an ergonomic builder for adding
//! common HTTP security headers to Sword applications.

pub use axum_helmet::{
    ContentSecurityPolicy, ContentSecurityPolicyDirective, CrossOriginEmbedderPolicy,
    CrossOriginOpenerPolicy, CrossOriginResourcePolicy, Header, HelmetLayer, OriginAgentCluster,
    ReferrerPolicy, StrictTransportSecurity, XContentTypeOptions, XDNSPrefetchControl,
    XDownloadOptions, XFrameOptions, XPermittedCrossDomainPolicies, XPoweredBy, XXSSProtection,
};

pub struct Helmet {
    headers: Vec<Header>,
}

impl Helmet {
    pub const fn builder() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    /// Adds a security header to the Helmet configuration.
    /// You can chain multiple calls to this method to add several headers.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    /// use sword::web::helmet::*;
    ///
    /// let helmet = Helmet::builder()
    ///     .with_header(XContentTypeOptions::nosniff())
    ///     .with_header(XXSSProtection::on())
    ///     .build();
    /// ```
    pub fn with_header<H: Into<Header>>(mut self, header: H) -> Self {
        self.headers.push(header.into());
        self
    }

    /// Builds the Helmet middleware layer.
    /// Once built, the layer can be added to the application using
    /// `ApplicationBuilder::with_layer()`.
    pub fn build(self) -> HelmetLayer {
        HelmetLayer::new(axum_helmet::Helmet {
            headers: self.headers,
        })
    }
}
