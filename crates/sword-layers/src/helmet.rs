//! Security headers middleware.
//!
//! This module wraps `axum-helmet` to provide an ergonomic builder for adding
//! common HTTP security headers to Sword applications.

use axum_helmet::Helmet as AxumHelmet;

pub use axum_helmet::{
    ContentSecurityPolicy, CrossOriginEmbedderPolicy, CrossOriginOpenerPolicy,
    CrossOriginResourcePolicy, Header, HelmetError, HelmetLayer, OriginAgentCluster,
    ReferrerPolicy, StrictTransportSecurity, XContentTypeOptions, XDNSPrefetchControl,
    XDownloadOptions, XFrameOptions, XPermittedCrossDomainPolicies, XPoweredBy, XXSSProtection,
};

pub struct Helmet {
    inner: AxumHelmet,
}

impl Helmet {
    pub fn builder() -> Self {
        Self {
            inner: AxumHelmet::new(),
        }
    }

    /// Adds a security header to the Helmet configuration.
    /// You can chain multiple calls to this method to add several headers.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword_layers::helmet::*;
    ///
    /// let helmet = Helmet::builder()
    ///     .with_header(XContentTypeOptions::nosniff())
    ///     .with_header(XXSSProtection::on())
    ///     .build()
    ///     .expect("failed to build helmet layer");
    /// ```
    pub fn with_header<H: Into<Header>>(mut self, header: H) -> Self {
        self.inner = self.inner.add(header);
        self
    }

    /// Builds the Helmet middleware layer.
    /// Once built, the layer can be added to the application using
    /// `ApplicationBuilder::with_layer()`.
    pub fn build(self) -> Result<HelmetLayer, HelmetError> {
        self.inner.into_layer()
    }
}
