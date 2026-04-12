#[cfg(any(feature = "web-controllers", feature = "socketio-controllers"))]
mod cmeta;
mod parse;

use proc_macro2::Span;
use syn::{
    Error, Ident, LitStr, Path, Token,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

#[cfg(any(feature = "web-controllers", feature = "socketio-controllers"))]
pub(crate) use cmeta::CMetaStack;
pub(crate) use parse::ControllerStruct;

pub enum ControllerKind {
    Web,
    SocketIo,
    Grpc,
}

#[derive(Default)]
pub struct ControllerArgs {
    pub kind: Option<ControllerKind>,
    pub path: Option<LitStr>,
    pub namespace: Option<LitStr>,
    pub service: Option<Path>,
}

pub enum ParsedControllerKind {
    #[cfg(feature = "web-controllers")]
    Web { path: String },

    #[cfg(feature = "socketio-controllers")]
    SocketIo { namespace: String },

    #[cfg(feature = "grpc-controllers")]
    Grpc { service: Path },
}

// Try to parse from a path like `Controller::Web` or `Controller::SocketIo`.
// This is a necesary heuristic to determine the controller kind without having access
// to the real `Controller` enum, which is not available at the time of parsing.
impl Parse for ControllerKind {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let path: Path = input.parse()?;

        let last = path
            .segments
            .last()
            .ok_or_else(|| Error::new(path.span(), "Expected a valid controller kind"))?
            .ident
            .to_string();

        match last.as_str() {
            "Web" => Ok(Self::Web),
            "SocketIo" => Ok(Self::SocketIo),
            "Grpc" => Ok(Self::Grpc),
            _ => Err(Error::new(
                path.span(),
                "Invalid controller kind. Expected `Web`, `SocketIo`, or `Grpc`",
            )),
        }
    }
}

// Parse arguments like `kind = Controller::Web, path = "/api"` from the attribute input.
// This allows us to support a flexible syntax for specifying controller arguments, while also
// providing good error messages for missing or invalid arguments.
impl Parse for ControllerArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut out = Self::default();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let key_span = key.span();
            let key_str = key.to_string();

            input.parse::<Token![=]>()?;

            match key_str.as_str() {
                "kind" => {
                    if out.kind.is_some() {
                        return Err(Error::new(key_span, "Duplicate argument `kind`"));
                    }
                    out.kind = Some(input.parse()?);
                }
                "path" => {
                    if out.path.is_some() {
                        return Err(Error::new(key_span, "Duplicate argument `path`"));
                    }
                    out.path = Some(input.parse()?);
                }
                "namespace" => {
                    if out.namespace.is_some() {
                        return Err(Error::new(key_span, "Duplicate argument `namespace`"));
                    }
                    out.namespace = Some(input.parse()?);
                }
                "service" => {
                    if out.service.is_some() {
                        return Err(Error::new(key_span, "Duplicate argument `service`"));
                    }
                    out.service = Some(input.parse()?);
                }
                _ => {
                    return Err(Error::new(key_span, "Unknown controller argument"));
                }
            }

            if input.is_empty() {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        Ok(out)
    }
}

// Convert the parsed `ControllerArgs` into a more specific `ParsedControllerKind`,
// validating the presence and format of required arguments based on the controller kind.
// This step is crucial for providing clear error messages when required arguments are missing or invalid,
// and for ensuring that the controller kind is correctly determined based on the provided arguments.
impl TryFrom<ControllerArgs> for ParsedControllerKind {
    type Error = syn::Error;

    fn try_from(args: ControllerArgs) -> Result<Self, Self::Error> {
        let kind = args
            .kind
            .ok_or_else(|| Error::new(Span::call_site(), "Missing required argument `kind`"))?;

        match kind {
            ControllerKind::Web => {
                if let Some(service) = args.service {
                    return Err(Error::new(service.span(), "`service` is not valid for Web"));
                }

                let path = args
                    .path
                    .ok_or_else(|| Error::new(Span::call_site(), "Web requires `path`"))?;

                if let Some(namespace) = args.namespace {
                    return Err(Error::new(
                        namespace.span(),
                        "`namespace` is not valid for Web",
                    ));
                }

                let path = path.value();

                if !path.starts_with('/') {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "Path must start with '/'",
                    ));
                }

                #[cfg(not(feature = "web-controllers"))]
                {
                    Err(Error::new(
                        Span::call_site(),
                        "Web controllers require enabling the `web-controllers` feature",
                    ))
                }

                #[cfg(feature = "web-controllers")]
                Ok(ParsedControllerKind::Web { path })
            }

            ControllerKind::SocketIo => {
                if let Some(service) = args.service {
                    return Err(Error::new(
                        service.span(),
                        "`service` is not valid for SocketIo",
                    ));
                }

                let namespace = args.namespace.ok_or_else(|| {
                    Error::new(Span::call_site(), "SocketIo requires `namespace`")
                })?;

                if let Some(path) = args.path {
                    return Err(Error::new(path.span(), "`path` is not valid for SocketIo"));
                }

                let namespace = namespace.value();

                if !namespace.starts_with('/') {
                    return Err(Error::new(
                        Span::call_site(),
                        "Namespace must start with '/'",
                    ));
                }

                #[cfg(not(feature = "socketio-controllers"))]
                {
                    Err(Error::new(
                        Span::call_site(),
                        "Socket.IO controllers require enabling the `socketio-controllers` feature",
                    ))
                }

                #[cfg(feature = "socketio-controllers")]
                Ok(ParsedControllerKind::SocketIo { namespace })
            }

            ControllerKind::Grpc => {
                if let Some(path) = args.path {
                    return Err(Error::new(path.span(), "`path` is not valid for Grpc"));
                }

                if let Some(namespace) = args.namespace {
                    return Err(Error::new(
                        namespace.span(),
                        "`namespace` is not valid for Grpc",
                    ));
                }

                #[cfg(not(feature = "grpc-controllers"))]
                {
                    let _ = args.service;
                    Err(Error::new(
                        Span::call_site(),
                        "gRPC controllers require enabling the `grpc-controllers` feature",
                    ))
                }

                #[cfg(feature = "grpc-controllers")]
                let service = args
                    .service
                    .ok_or_else(|| Error::new(Span::call_site(), "Grpc requires `service`"))?;

                #[cfg(feature = "grpc-controllers")]
                Ok(ParsedControllerKind::Grpc { service })
            }
        }
    }
}
