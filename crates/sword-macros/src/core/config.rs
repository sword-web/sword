use crate::shared::{Result, location, paths::*};

use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

// `#[config(key = "literal_str")]` macro Args
#[derive(FromMeta)]
struct Args {
    key: String,
}

pub fn expand_config_struct(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let input = syn::parse::<DeriveInput>(item)?;
    let args = Args::from_list(&NestedMeta::parse_meta_list(args.into())?)?;

    let config_key = args.key;
    let struct_name = &input.ident;
    let self_ty = quote! { #struct_name };
    let location = location::tokens(struct_name.span(), &struct_name.to_string());

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

                config.get::<Self>()
                    .ok_or_else(|| #DependencyInjectionError::dependency_not_found(format!("Config item '{}'", Self::key())))
            }
        }

        // Auto-register this config type using inventory
        // This allows automatic registration of all configs during app initialization
        // We use the full path to inventory through sword's re-export
        const _: () = {
            #inventory::submit! {
                #ConfigRegistrar::new(|state, config| {
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
                                message: format!("config item `{}` was not found", #config_key),
                            },
                            context: {
                                "key" => #config_key,
                            },
                            hints: ["Ensure the configuration file contains this section"],
                        });
                    };

                    state.insert(config_item);

                    Ok(())
                })
            }
        };
    };

    Ok(TokenStream::from(expanded))
}
