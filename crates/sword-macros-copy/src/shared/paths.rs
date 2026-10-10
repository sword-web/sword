macro_rules! path {
    ($name:ident => $($path:tt)*) => {
        #[allow(non_camel_case_types)]
        pub struct $name;
        impl quote::ToTokens for $name {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                tokens.extend(quote::quote!($($path)*));
            }
        }
    };
}

path!(State => ::sword::internal::core::State);

path!(Config => ::sword::internal::core::Config);
path!(ConfigItem => ::sword::internal::core::ConfigItem);
path!(ConfigRegistrar => ::sword::internal::core::ConfigRegistrar);

path!(DependencyInjectionError => ::sword::internal::core::DependencyInjectionError);

path!(inventory => ::sword::internal::inventory);
path!(sword_error => ::sword::internal::core::sword_error);
