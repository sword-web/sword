macro_rules! path {
    ($name:ident => $($path:tt)*) => {
        #[allow(non_camel_case_types, dead_code)]
        pub struct $name;
        impl quote::ToTokens for $name {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                tokens.extend(quote::quote!($($path)*));
            }
        }
    };
}

path!(TypeId => ::std::any::TypeId);
path!(Box => ::std::boxed::Box);
path!(Arc => ::std::sync::Arc);

path!(State => ::sword::internal::core::State);
path!(Component => ::sword::internal::core::Component);
path!(Provider => ::sword::internal::core::Provider);

path!(Controller => ::sword::internal::core::Controller);
path!(ControllerSpec => ::sword::internal::core::ControllerSpec);

path!(MethodRouter => ::sword::internal::web::MethodRouter);
path!(WebController => ::sword::internal::web::WebController);
path!(WebControllerRegistrar => ::sword::internal::web::WebControllerRegistrar);
path!(RouteRegistrar => ::sword::internal::web::RouteRegistrar);

path!(Request => ::sword::web::Request);
path!(OnRequest => ::sword::web::OnRequest);
path!(OnRequestWithConfig => ::sword::web::OnRequestWithConfig);
path!(Next => ::sword::web::Next);

path!(mw_with_state => ::sword::internal::web::mw_with_state);
path!(routing => ::sword::internal::web::routing);
path!(IntoResponse => ::sword::internal::web::IntoResponse);
path!(AxumResponse => ::sword::internal::web::AxumResponse);
path!(JsonResponse => ::sword::web::JsonResponse);

path!(StreamRequest => ::sword::web::StreamRequest);

path!(OnRequestStream => ::sword::web::OnRequestStream);
path!(OnRequestStreamWithConfig => ::sword::web::OnRequestStreamWithConfig);

path!(Event => ::sword::internal::events::Event);

path!(Interceptor => ::sword::internal::core::Interceptor);
path!(InterceptorRegistrar => ::sword::internal::core::InterceptorRegistrar);

path!(Build => ::sword::internal::core::Build);
path!(HasDeps => ::sword::internal::core::HasDeps);

path!(Config => ::sword::internal::core::Config);
path!(ConfigItem => ::sword::internal::core::ConfigItem);
path!(ConfigRegistrar => ::sword::internal::core::ConfigRegistrar);

path!(DependencyInjectionError => ::sword::internal::core::DependencyInjectionError);

path!(inventory => ::sword::internal::inventory);
path!(sword_error => ::sword::internal::core::sword_error);
path!(tracing => ::sword::internal::tracing);

path!(Job => ::sword::internal::jobs::Job);
path!(JobFunctionRegistrar => ::sword::internal::jobs::JobFunctionRegistrar);
path!(JobControllerRegistrar => ::sword::internal::jobs::JobControllerRegistrar);
