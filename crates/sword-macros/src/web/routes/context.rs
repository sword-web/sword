use crate::{core::InterceptorArgs, shared::cmeta::CMetaStack};
use proc_macro2::Span;

pub struct WebRouteContext {
    pub controller_name: String,
    pub controller_interceptors: Vec<InterceptorArgs>,
}

impl WebRouteContext {
    pub fn get() -> syn::Result<Self> {
        let Some(controller_name) = CMetaStack::get("web", "controller_name") else {
            let error = "\n[ERROR] This attribute macro must be used inside a Web Controller impl block.\n\
                \n\
                Make sure:\n\
                1. The struct has #[controller(kind = Controller::Web, path = \"/path\")] attribute\n\
                2. The struct is defined BEFORE the impl block\n\
                3. The impl block is for the same struct\n";

            return Err(syn::Error::new(Span::call_site(), error));
        };

        let controller_interceptors = CMetaStack::get_list("web", "controller_interceptors")
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
