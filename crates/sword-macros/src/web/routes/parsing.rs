use crate::{
    core::{InterceptorArgs, take_interceptors},
    shared::{Result, paths::*},
};

use super::WebRouteContext;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{GenericArgument, ItemFn, LitStr, PathArguments, ReturnType, Type, spanned::Spanned};

/// Parsed data from a route attribute
pub struct ParsedRouteAttribute {
    /// The HTTP method (GET, POST, etc.)
    pub method: HttpMethod,

    /// The route path (e.g., "/", "/:id")
    pub path: String,

    /// The original function, kept as-is.
    pub function: ItemFn,

    /// Parsed interceptor arguments
    pub interceptors: Vec<InterceptorArgs>,

    /// Request extraction mode inferred from function signature.
    pub request_mode: RequestMode,

    /// Controller metadata from CMetaStack
    pub context: WebRouteContext,

    /// Classified handler return metadata
    pub return_info: RouteReturnInfo,
}

impl ParsedRouteAttribute {
    pub fn parse(method: &str, attr: TokenStream, item: TokenStream) -> Result<Self> {
        let input_fn = syn::parse::<ItemFn>(item)?;
        let method = HttpMethod::from_attr_name(method)?;

        let path = syn::parse::<RouteArgs>(attr)?.path.value();
        let request_mode = RequestMode::try_from(&input_fn)?;

        let interceptors = take_interceptors(&input_fn.attrs)?;
        let context = WebRouteContext::get()?;
        let return_info = RouteReturnInfo::from_signature(&input_fn.sig, method);

        Ok(Self {
            method,
            path,
            function: input_fn,
            interceptors,
            request_mode,
            context,
            return_info,
        })
    }

    pub fn route_fn_name(&self) -> syn::Ident {
        format_ident!("__sword_web_route_{}", self.function.sig.ident)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnKind {
    Passthrough,
    Serialize,
    Empty,
}

/// How a route handler return its response
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RouteReturnInfo {
    pub is_result: bool,
    pub kind: ReturnKind,
    pub status_code: u16,
}

impl RouteReturnInfo {
    pub fn from_signature(sig: &syn::Signature, method: HttpMethod) -> Self {
        let status_code = method.default_status_code();
        let plain = Self {
            status_code,
            is_result: false,
            kind: ReturnKind::Passthrough,
        };

        let Some(ok_type) = Self::result_ok_type(sig) else {
            return plain;
        };

        Self {
            is_result: true,
            kind: Self::classify_return_kind(&ok_type),
            status_code,
        }
    }

    /// Extracts `T` from `Result<T, E>` / `WebResult<T, E>`
    /// or `WebResult` without generics, which means `JsonResponse`.
    ///
    /// Returns `None` for anything that is not a `Result`/`WebResult` return.
    fn result_ok_type(sig: &syn::Signature) -> Option<Type> {
        let return_type = match &sig.output {
            ReturnType::Type(_, ty) => ty.as_ref(),
            _ => return None,
        };

        let Type::Path(type_path) = return_type else {
            return None;
        };

        // Gets the last segment for a type. e.g ::std::sync::Arc -> Arc
        let last_segment = type_path.path.segments.last()?;
        let ty_str = last_segment.ident.to_string();

        match ty_str.as_str() {
            "Result" => {
                let args = match &last_segment.arguments {
                    PathArguments::AngleBracketed(args) => &args.args,
                    _ => return None,
                };

                if args.len() < 2 {
                    return None;
                }

                match &args[0] {
                    GenericArgument::Type(ty) => Some(ty.clone()),
                    _ => None,
                }
            }

            "WebResult" => match &last_segment.arguments {
                // WebResult<T, E> -> T
                PathArguments::AngleBracketed(args) if !args.args.is_empty() => {
                    match &args.args[0] {
                        GenericArgument::Type(ty) => Some(ty.clone()),
                        _ => None,
                    }
                }

                // Empty WebResult == WebResult<JsonResponse>
                _ => Some(syn::parse_quote!(JsonResponse)),
            },

            _ => None,
        }
    }

    fn classify_return_kind(ty: &Type) -> ReturnKind {
        if let Type::Tuple(tuple) = ty
            && tuple.elems.is_empty()
        {
            return ReturnKind::Empty;
        }

        if let Type::Path(type_path) = ty {
            // Gets the last segment for a type. e.g ::std::sync::Arc -> Arc
            let Some(last_segment) = type_path.path.segments.last() else {
                return ReturnKind::Serialize;
            };

            let ty_str = last_segment.ident.to_string();

            match ty_str.as_str() {
                "JsonResponse" | "File" | "Redirect" | "Sse" => return ReturnKind::Passthrough,
                _ => return ReturnKind::Serialize,
            }
        }

        ReturnKind::Serialize
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RequestMode {
    None,
    Buffered,
    Streaming,
}

impl RequestMode {
    pub fn to_tokens(self) -> Option<TokenStream2> {
        match self {
            Self::Buffered => Some(quote! { #Request }),
            Self::Streaming => Some(quote! { #StreamRequest }),
            Self::None => None,
        }
    }
}

impl TryFrom<&ItemFn> for RequestMode {
    type Error = darling::Error;

    fn try_from(input_fn: &ItemFn) -> std::result::Result<Self, Self::Error> {
        let mut mode = Self::None;

        for arg in &input_fn.sig.inputs {
            let syn::FnArg::Typed(pat_type) = arg else {
                continue;
            };

            let arg_mode = match &*pat_type.ty {
                Type::Path(type_path) => match type_path.path.segments.last() {
                    Some(segment) if segment.ident == "Request" => Self::Buffered,
                    Some(segment) if segment.ident == "StreamRequest" => Self::Streaming,
                    _ => Self::None,
                },
                _ => Self::None,
            };

            if arg_mode == Self::None {
                continue;
            }

            if mode != Self::None && mode != arg_mode {
                return Err(darling::Error::custom(
                    "A route handler cannot use both `Request` and `StreamRequest` in the same signature",
                ).with_span(&pat_type.span()));
            }

            mode = arg_mode;
        }

        Ok(mode)
    }
}

pub struct RouteArgs {
    pub path: LitStr,
}

impl syn::parse::Parse for RouteArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Route path is required, e.g., #[get(\"/path\")]. If you want the root path, use \"/\".",
            ));
        }

        let path: LitStr = input.parse()?;

        Ok(Self { path })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Trace,
    Connect,
    Sse,
}

impl HttpMethod {
    pub fn from_attr_name(method: &str) -> syn::Result<Self> {
        match method {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            "HEAD" => Ok(Self::Head),
            "OPTIONS" => Ok(Self::Options),
            "TRACE" => Ok(Self::Trace),
            "CONNECT" => Ok(Self::Connect),
            "SSE" => Ok(Self::Sse),
            _ => Err(syn::Error::new(
                Span::call_site(),
                format!("Unsupported HTTP method `{method}`"),
            )),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Trace => "TRACE",
            Self::Connect => "CONNECT",
            Self::Sse => "SSE",
        }
    }

    pub fn routing_fn_tokens(self) -> proc_macro2::TokenStream {
        match self {
            Self::Post => quote! { post },
            Self::Put => quote! { put },
            Self::Delete => quote! { delete },
            Self::Patch => quote! { patch },
            Self::Head => quote! { head },
            Self::Options => quote! { options },
            Self::Trace => quote! { trace },
            Self::Connect => quote! { connect },
            Self::Sse | Self::Get => quote! { get },
        }
    }

    pub const fn default_status_code(self) -> u16 {
        match self {
            Self::Post => 201,
            _ => 200,
        }
    }

    /// Whether the route is a Server-Sent Events stream.
    ///
    /// SSE routes are served over GET but must remain streaming responses, so the
    /// generator treats them as passthrough regardless of the return type.
    pub const fn is_sse(self) -> bool {
        matches!(self, Self::Sse)
    }
}
