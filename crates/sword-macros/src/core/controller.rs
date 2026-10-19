use crate::{
    core::{InterceptorArgs, take_interceptors},
    shared::{Error, Result, paths::*, structs::Struct},
};

use darling::{FromDeriveInput, FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Ident, LitStr, Path, Type};

// `#[controller(kind = ..., ...)]` model =====================================================

pub enum ControllerKind {
    Web { path: String },
    SocketIo { namespace: String },
    Grpc { service: Path },
    Events { source: EventSourceKind },
    Jobs,
}

pub enum EventSourceKind {
    Memory,
    Nats,
}

impl EventSourceKind {
    pub fn as_tokens(&self) -> TokenStream2 {
        match self {
            Self::Memory => quote! { #EventSource::Memory },
            Self::Nats => quote! { #EventSource::Nats },
        }
    }
}

#[derive(FromMeta)]
struct ControllerArgs {
    kind: Path,

    #[darling(default)]
    path: Option<LitStr>,

    #[darling(default)]
    namespace: Option<LitStr>,

    #[darling(default)]
    service: Option<Path>,

    #[darling(default)]
    source: Option<Path>,
}

pub struct ControllerStruct {
    base: Struct,
    pub kind: ControllerKind,
    pub interceptors: Vec<InterceptorArgs>,
}

impl ControllerStruct {
    pub fn fields(&self) -> Vec<(Ident, Type)> {
        self.base.fields()
    }

    pub const fn ty(&self) -> &Ident {
        &self.base.ident
    }

    pub fn name(&self) -> String {
        self.ty().to_string()
    }

    pub fn span(&self) -> Span {
        self.ty().span()
    }
}

// args -> kind ==============================================================================

impl ControllerArgs {
    fn resolve_kind(self) -> Result<ControllerKind> {
        let name = self
            .kind
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default();

        match name.as_str() {
            "Web" => {
                if let Some(service) = &self.service {
                    return Err(Error::custom("`service` is not valid for Web").with_span(service));
                }

                if let Some(namespace) = &self.namespace {
                    return Err(
                        Error::custom("`namespace` is not valid for Web").with_span(namespace)
                    );
                }

                if let Some(source) = &self.source {
                    return Err(Error::custom("`source` is not valid for Web").with_span(source));
                }

                let path = self
                    .path
                    .as_ref()
                    .ok_or_else(|| Error::custom("Web requires `path`").with_span(&self.kind))?
                    .value();

                if !path.starts_with('/') {
                    return Err(Error::custom("path must start with '/'").with_span(&self.kind));
                }

                #[cfg(feature = "web")]
                {
                    Ok(ControllerKind::Web { path })
                }

                #[cfg(not(feature = "web"))]
                {
                    Err(
                        Error::custom("Web controllers require enabling the `web` feature")
                            .with_span(&self.kind),
                    )
                }
            }

            "SocketIo" => {
                if let Some(path) = &self.path {
                    return Err(Error::custom("`path` is not valid for SocketIo").with_span(path));
                }

                if let Some(service) = &self.service {
                    return Err(
                        Error::custom("`service` is not valid for SocketIo").with_span(service)
                    );
                }

                if let Some(source) = &self.source {
                    return Err(
                        Error::custom("`source` is not valid for SocketIo").with_span(source)
                    );
                }

                let namespace = self
                    .namespace
                    .as_ref()
                    .ok_or_else(|| {
                        Error::custom("SocketIo requires `namespace`").with_span(&self.kind)
                    })?
                    .value();

                if !namespace.starts_with('/') {
                    return Err(
                        Error::custom("namespace must start with '/'").with_span(&self.kind)
                    );
                }

                #[cfg(feature = "socketio")]
                {
                    Ok(ControllerKind::SocketIo { namespace })
                }

                #[cfg(not(feature = "socketio"))]
                {
                    Err(Error::custom(
                        "SocketIo controllers require enabling the `socketio` feature",
                    )
                    .with_span(&self.kind))
                }
            }

            "Grpc" => {
                if let Some(path) = &self.path {
                    return Err(Error::custom("`path` is not valid for Grpc").with_span(path));
                }

                if let Some(namespace) = &self.namespace {
                    return Err(
                        Error::custom("`namespace` is not valid for Grpc").with_span(namespace)
                    );
                }

                if let Some(source) = &self.source {
                    return Err(Error::custom("`source` is not valid for Grpc").with_span(source));
                }

                let service = self.service.as_ref().ok_or_else(|| {
                    Error::custom("Grpc requires `service`").with_span(&self.kind)
                })?;

                #[cfg(feature = "grpc")]
                {
                    Ok(ControllerKind::Grpc {
                        service: service.clone(),
                    })
                }

                #[cfg(not(feature = "grpc"))]
                {
                    let _ = service;

                    Err(
                        Error::custom("Grpc controllers require enabling the `grpc` feature")
                            .with_span(&self.kind),
                    )
                }
            }

            "Events" => {
                if let Some(path) = &self.path {
                    return Err(
                        Error::custom("`path` is not valid for Events Controllers").with_span(path)
                    );
                }

                if let Some(namespace) = &self.namespace {
                    return Err(
                        Error::custom("`namespace` is not valid for Events Controllers")
                            .with_span(namespace),
                    );
                }

                if let Some(service) = &self.service {
                    return Err(
                        Error::custom("`service` is not valid for Events Controllers")
                            .with_span(service),
                    );
                }

                let source = self.source.as_ref().ok_or_else(|| {
                    Error::custom("Events Controller requires `source`").with_span(&self.kind)
                })?;

                let source_name = source
                    .segments
                    .last()
                    .map(|segment| segment.ident.to_string())
                    .unwrap_or_default();

                match source_name.as_str() {
                    "Memory" => {
                        #[cfg(feature = "events")]
                        {
                            Ok(ControllerKind::Events {
                                source: EventSourceKind::Memory,
                            })
                        }

                        #[cfg(not(feature = "events"))]
                        {
                            Err(Error::custom(
                                "Events controllers require enabling the `events` feature",
                            )
                            .with_span(&self.kind))
                        }
                    }

                    other => {
                        Err(Error::custom(format!("unknown event source `{other}`"))
                            .with_span(source))
                    }
                }
            }

            "Jobs" => {
                if let Some(path) = &self.path {
                    return Err(
                        Error::custom("`path` is not valid for Jobs Controller").with_span(path)
                    );
                }

                if let Some(namespace) = &self.namespace {
                    return Err(
                        Error::custom("`namespace` is not valid for Jobs Controller")
                            .with_span(namespace),
                    );
                }

                if let Some(service) = &self.service {
                    return Err(Error::custom("`service` is not valid for Jobs Controller")
                        .with_span(service));
                }

                if let Some(source) = &self.source {
                    return Err(Error::custom("`source` is not valid for Jobs Controllers")
                        .with_span(source));
                }

                #[cfg(feature = "jobs")]
                {
                    Ok(ControllerKind::Jobs)
                }

                #[cfg(not(feature = "jobs"))]
                {
                    Err(
                        Error::custom("Jobs controllers require enabling the `jobs` feature")
                            .with_span(&self.kind),
                    )
                }
            }

            other => {
                Err(Error::custom(format!("unknown controller kind `{other}`"))
                    .with_span(&self.kind))
            }
        }
    }
}

// expansion =================================================================================

pub fn expand_controller(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let derive_input = syn::parse::<DeriveInput>(item.clone())?;
    let base = Struct::from_derive_input(&derive_input)?;

    let args = ControllerArgs::from_list(&NestedMeta::parse_meta_list(attr.into())?)?;
    let kind = args.resolve_kind()?;

    let interceptors = take_interceptors(&derive_input.attrs)?;

    let parsed = ControllerStruct {
        base,
        kind,
        interceptors,
    };

    let builder: TokenStream2 = match &parsed.kind {
        #[cfg(feature = "web")]
        ControllerKind::Web { .. } => crate::web::expand_controller(&parsed),

        #[cfg(feature = "socketio")]
        ControllerKind::SocketIo { .. } => crate::socketio::expand_controller(&parsed),

        #[cfg(feature = "grpc")]
        ControllerKind::Grpc { .. } => crate::grpc::expand_controller(&parsed),

        #[cfg(feature = "events")]
        ControllerKind::Events { .. } => crate::events::expand_controller(&parsed),

        #[cfg(feature = "jobs")]
        ControllerKind::Jobs => crate::jobs::expand_controller(&parsed),

        #[allow(unreachable_patterns)]
        _ => Err(Error::custom("no controller kind feature is enabled")),
    }?;

    let struct_tokens: TokenStream2 = item.into();

    Ok(quote! {
        #struct_tokens
        #builder
    }
    .into())
}
