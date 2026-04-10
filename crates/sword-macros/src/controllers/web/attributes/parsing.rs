use crate::{controllers::shared::CMetaStack, interceptor::InterceptorArgs};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, ItemFn, LitStr, Type};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RequestMode {
    None,
    Buffered,
    Streaming,
}

#[derive(Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl HttpMethod {
    pub fn from_attr_name(method: &str) -> syn::Result<Self> {
        match method {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            _ => Err(syn::Error::new(
                Span::call_site(),
                format!("Unsupported HTTP method `{method}`"),
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
        }
    }

    pub fn routing_fn_tokens(self) -> TokenStream2 {
        match self {
            Self::Get => quote! { get },
            Self::Post => quote! { post },
            Self::Put => quote! { put },
            Self::Delete => quote! { delete },
            Self::Patch => quote! { patch },
        }
    }
}

pub struct WebRouteContext {
    pub controller_name: String,
    pub controller_interceptors: Vec<InterceptorArgs>,
}

impl WebRouteContext {
    fn from_cmeta(method: &str) -> syn::Result<Self> {
        let Some(controller_name) = CMetaStack::get("controller_name") else {
            let error = format!(
                "\n[ERROR] The #[{}] attribute must be used inside a #[controller] impl block.\n\
                \n\
                Make sure:\n\
                1. The struct has #[controller(kind = Controller::Web, path = \"/path\")] attribute\n\
                2. The struct is defined BEFORE the impl block\n\
                3. The impl block is for the same struct\n",
                method
            );

            return Err(syn::Error::new(Span::call_site(), error));
        };

        let controller_interceptors = CMetaStack::get_list("controller_interceptors")
            .unwrap_or_default()
            .into_iter()
            .map(|interceptor| syn::parse_str::<InterceptorArgs>(&interceptor))
            .collect::<syn::Result<Vec<_>>>()?;

        Ok(Self {
            controller_name,
            controller_interceptors,
        })
    }
}

/// Parsed data from a route attribute
pub struct ParsedRouteAttribute {
    /// The HTTP method (GET, POST, etc.)
    pub method: HttpMethod,

    /// The route path (e.g., "/", "/:id")
    pub path: String,

    /// The original function with interceptor attributes removed
    pub function: ItemFn,

    /// Parsed interceptor arguments
    pub interceptors: Vec<InterceptorArgs>,

    /// Request extraction mode inferred from function signature.
    pub request_mode: RequestMode,

    /// Controller metadata from CMetaStack
    pub context: WebRouteContext,
}

impl ParsedRouteAttribute {
    pub fn parse(method: &str, attr: TokenStream, item: TokenStream) -> syn::Result<Self> {
        let method = HttpMethod::from_attr_name(method)?;
        let path = Self::parse_path(attr)?;
        let mut input_fn = Self::parse_function(item)?;
        let request_mode = Self::infer_request_mode(&input_fn)?;

        let (interceptors, retained_attrs) = Self::extract_interceptors(&input_fn)?;
        input_fn.attrs = retained_attrs;

        let context = Self::resolve_context(method.as_str())?;

        Ok(Self {
            method,
            path,
            function: input_fn,
            interceptors,
            request_mode,
            context,
        })
    }

    fn parse_path(attr: TokenStream) -> syn::Result<String> {
        if attr.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Route path is required, e.g., #[get(\"/path\")]. If you want the root path, use \"/\".",
            ));
        }

        Ok(syn::parse::<LitStr>(attr)?.value())
    }

    fn parse_function(item: TokenStream) -> syn::Result<ItemFn> {
        syn::parse::<ItemFn>(item)
    }

    fn extract_interceptors(
        input_fn: &ItemFn,
    ) -> syn::Result<(Vec<InterceptorArgs>, Vec<Attribute>)> {
        let mut interceptors = Vec::new();
        let mut retained_attrs = Vec::new();

        for attr in input_fn.attrs.iter() {
            if attr.path().is_ident("interceptor") {
                interceptors.push(attr.parse_args::<InterceptorArgs>()?);
            } else {
                retained_attrs.push(attr.clone());
            }
        }

        Ok((interceptors, retained_attrs))
    }

    fn resolve_context(method: &str) -> syn::Result<WebRouteContext> {
        WebRouteContext::from_cmeta(method)
    }

    fn infer_request_mode(input_fn: &ItemFn) -> syn::Result<RequestMode> {
        let mut mode = RequestMode::None;

        for arg in &input_fn.sig.inputs {
            let syn::FnArg::Typed(pat_type) = arg else {
                continue;
            };

            let arg_mode = Self::request_mode_from_type(&pat_type.ty);

            if arg_mode == RequestMode::None {
                continue;
            }

            if mode != RequestMode::None && mode != arg_mode {
                return Err(syn::Error::new(
                    pat_type.ty.span(),
                    "A route handler cannot use both `Request` and `StreamRequest` in the same signature",
                ));
            }

            mode = arg_mode;
        }

        Ok(mode)
    }

    fn request_mode_from_type(ty: &Type) -> RequestMode {
        let Type::Path(type_path) = ty else {
            return RequestMode::None;
        };

        let Some(last_segment) = type_path.path.segments.last() else {
            return RequestMode::None;
        };

        if last_segment.ident == "Request" {
            return RequestMode::Buffered;
        }

        if last_segment.ident == "StreamRequest" {
            return RequestMode::Streaming;
        }

        RequestMode::None
    }
}
