use super::shared::{ControllerStruct, ParsedControllerKind};
use crate::shared::{gen_build, gen_clone, gen_deps};

use proc_macro::TokenStream;
use quote::quote;
use syn::Error;

pub fn expand_grpc_controller(input: &ControllerStruct) -> syn::Result<TokenStream> {
    let ParsedControllerKind::Grpc { service } = &input.kind else {
        return Err(Error::new(input.name.span(), "Expected a gRPC controller"));
    };

    let self_name = &input.name;
    let self_fields = &input.fields;
    let interceptors = &input.interceptors;

    let service_name = service
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_else(|| "grpc_service".to_string());

    let deps_impl = gen_deps(self_name, self_fields);
    let build_impl = gen_build(self_name, self_fields);
    let clone_impl = gen_clone(self_name, self_fields);

    #[cfg(feature = "grpc-reflection")]
    let reflection_descriptor_set = quote! {
        Some(include_bytes!(concat!(env!("OUT_DIR"), "/sword_descriptor_set.bin")))
    };

    #[cfg(not(feature = "grpc-reflection"))]
    let reflection_descriptor_set = quote! { None };

    let interceptor_wrappers = interceptors.iter().rev().map(|interceptor| match interceptor {
        crate::interceptor::InterceptorArgs::Traditional(path) => {
            quote! {
                let interceptor = state.borrow::<#path>()
                    .unwrap_or_else(|err| {
                        ::sword::internal::core::sword_error! {
                            title: "Failed to retrieve gRPC interceptor from State",
                            reason: err,
                            context: {
                                "controller" => stringify!(#self_name),
                                "interceptor" => stringify!(#path),
                            },
                            hints: ["Ensure the interceptor is registered and built before gRPC controller setup"],
                        }
                    });

                let service = ::sword::internal::grpc::tonic_async_interceptor::AsyncInterceptedService::new(
                    service,
                    move |req: ::sword::internal::grpc::tonic::Request<()>| {
                        let interceptor = ::std::sync::Arc::clone(&interceptor);
                        async move {
                            <#path as ::sword::grpc::OnRequest>::on_request(&*interceptor, req).await
                        }
                    },
                );
            }
        }
        crate::interceptor::InterceptorArgs::WithConfig {
            interceptor,
            config,
        } => {
            quote! {
                let interceptor = state.borrow::<#interceptor>()
                    .unwrap_or_else(|err| {
                        ::sword::internal::core::sword_error!(
                            title: "Failed to retrieve gRPC interceptor from State",
                            reason: err,
                            context: {
                                "controller" => stringify!(#self_name),
                                "interceptor" => stringify!(#interceptor),
                            },
                            hints: ["Ensure the interceptor is registered and built before gRPC controller setup"],
                        )
                    });

                let service = ::sword::internal::grpc::tonic_async_interceptor::AsyncInterceptedService::new(
                    service,
                    move |req: ::sword::internal::grpc::tonic::Request<()>| {
                        let interceptor = ::std::sync::Arc::clone(&interceptor);
                        async move {
                            <#interceptor as ::sword::grpc::OnRequestWithConfig<_>>::on_request(
                                &*interceptor,
                                #config,
                                req,
                            ).await
                        }
                    },
                );
            }
        }
        crate::interceptor::InterceptorArgs::Expression(expr) => {
            quote! {
                let service = ::sword::internal::grpc::tonic_async_interceptor::AsyncInterceptedService::new(
                    service,
                    #expr,
                );
            }
        }
    });

    let expanded = quote! {
        #build_impl
        #deps_impl
        #clone_impl

        ::sword::internal::inventory::submit! {
            ::sword::internal::grpc::GrpcControllerRegistrar {
                controller_id: ::std::any::TypeId::of::<#self_name>(),
                service_name: stringify!(#service),
                reflection_descriptor_set: #reflection_descriptor_set,
                build: |state: &::sword::internal::core::State| {
                    state.insert::<#self_name>(#self_name::build(state).unwrap_or_else(|e| {
                        ::sword::internal::core::sword_error! {
                            title: "Failed to build gRPC controller",
                            reason: "An error occurred while building the gRPC controller",
                            context: {
                                "controller_name" => stringify!(#self_name),
                                "service" => stringify!(#service),
                                "error" => format!("{e:?}"),
                                "source" => "GrpcControllerRegistrar::build",
                            },
                            hints: ["Check the error message for details on what went wrong during construction"],
                        }
                    }));
                },
                register: |_state: &::sword::internal::core::State, _registry: &mut ::sword::internal::grpc::GrpcServiceRegistry| {
                    let state = _state;
                    let registry = _registry;

                    let controller = state.borrow::<#self_name>().unwrap_or_else(|err| {
                        ::sword::internal::core::sword_error! {
                            title: "Failed to retrieve gRPC controller from State",
                            reason: err,
                            context: {
                                "controller_name" => stringify!(#self_name),
                                "service" => stringify!(#service),
                                "source" => "GrpcControllerRegistrar::register",
                            },
                            hints: ["Ensure the controller is built before registration"],
                        }
                    });

                    let body_limit = state.borrow::<::sword::internal::grpc::GrpcBodyLimitValue>()
                        .unwrap_or_else(|err| {
                            ::sword::internal::core::sword_error!(
                                title: "Failed to retrieve gRPC body-limit settings from State",
                                reason: err,
                                context: {
                                    "controller_name" => stringify!(#self_name),
                                    "service" => stringify!(#service),
                                    "source" => "GrpcControllerRegistrar::register",
                                },
                                hints: ["Ensure gRPC body-limit settings are inserted into State before controller registration"],
                            )
                        });

                    let service = <#service<#self_name>>::new((*controller).clone())
                        .max_decoding_message_size(body_limit.max_decoding_message_size)
                        .max_encoding_message_size(body_limit.max_encoding_message_size);
                    #(#interceptor_wrappers)*
                    registry.routes_builder_mut().add_service(service);
                    registry.mark_service_registered_with_name(<#service<#self_name> as ::sword::internal::grpc::tonic::server::NamedService>::NAME);
                },
            }
        }

        impl ::sword::internal::grpc::GrpcController for #self_name {
            fn service_name() -> &'static str {
                #service_name
            }
        }

        impl ::sword::internal::core::ControllerSpec for #self_name {
            fn kind() -> ::sword::internal::core::Controller {
                ::sword::internal::core::Controller::Grpc
            }

            fn type_id() -> ::std::any::TypeId {
                ::std::any::TypeId::of::<#self_name>()
            }
        }
    };

    Ok(TokenStream::from(expanded))
}
