use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::shared::{Result, location, paths::*};

// `#[config(key = "literal_str", required = bool)]` macro Args
#[derive(FromMeta)]
struct Args {
    key: String,
    #[darling(default)]
    required: Option<bool>,
}

pub fn expand_config_struct(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let input = syn::parse::<DeriveInput>(item)?;
    let args = Args::from_list(&NestedMeta::parse_meta_list(args.into())?)?;

    let config_key = args.key;
    let required = args.required.unwrap_or(true);
    let struct_name = &input.ident;
    let self_ty = quote! { #struct_name };
    let location = location::tokens(struct_name.span(), &struct_name.to_string());

    let load_config = if required {
        quote! {
            let config_item = config.try_get::<#self_ty>().map_err(|err| {
                #sword_error! {
                    code: "sword.build.config",
                    phase: #DiagnosticPhase::Build,
                    target: "config registry",
                    title: "Failed to load registered config item",
                    reason: err,
                    source_location: {
                        file: (#location).file(),
                        line: (#location).line(),
                        column: (#location).column(),
                        length: (#location).length(),
                        message: format!("config item `{}`", #config_key),
                    },
                    context: {
                        "key" => #config_key,
                    },
                    hints: ["Check the types of the values in this section"],
                    cause: err,
                }
            })?;

            let Some(config_item) = config_item else {
                return Err(#sword_error! {
                    code: "sword.build.config",
                    phase: #DiagnosticPhase::Build,
                    target: "config registry",
                    title: "Config item not found",
                    reason: format!("Config item '{}' was not found", #config_key),
                    source_location: {
                        file: (#location).file(),
                        line: (#location).line(),
                        column: (#location).column(),
                        length: (#location).length(),
                        message: format!("`{}`", #config_key),
                    },
                    context: {
                        "key" => #config_key,
                    },
                    hints: [format!("Add a `[{}]` table to the configuration file", #config_key)],
                });
            };
        }
    } else {
        quote! {
            let config_item = config.get_or_default::<#self_ty>();
        }
    };

    let try_from_config = if required {
        quote! {
            config
                .get::<Self>()
                .ok_or_else(|| #DependencyInjectionError::dependency_not_found(format!(
                    "Config item '{}'",
                    <Self as #ConfigItem>::key()
                )))
        }
    } else {
        quote! {
            Ok(config.get_or_default::<Self>())
        }
    };

    let expanded = quote! {
        #input

        impl #ConfigItem for #struct_name {
            fn key() -> &'static str {
                #config_key
            }
        }

        impl TryFrom<&#State> for #struct_name {
            type Error = #DependencyInjectionError;

            fn try_from(state: &#State) -> Result<Self, Self::Error> {
                let config = state.get::<#Config>()
                    .map_err(|_| #DependencyInjectionError::dependency_not_found("Config"))?;

                #try_from_config
            }
        }

        const _: () = {
            #inventory::submit! {
                #ConfigRegistrar::new(|state, config| {
                    #load_config
                    state.insert(config_item);

                    Ok(())
                })
            }
        };
    };

    Ok(TokenStream::from(expanded))
}
