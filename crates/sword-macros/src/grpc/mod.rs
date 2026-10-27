mod errors;

pub use errors::derive_grpc_error;

use crate::{
    core::{ControllerKind, ControllerStruct, InterceptorArgs},
    shared::{
        Result, location,
        paths::*,
        traits::{generate_build_impl, generate_clone_impl},
    },
};

use darling::Error;
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_controller(controller: &ControllerStruct) -> Result<TokenStream> {
    let ControllerKind::Grpc { service } = &controller.kind else {
        return Err(Error::custom("Expected a gRPC controller").with_span(&controller.span()));
    };

    let self_ty = controller.ty();
    let location = location::tokens(self_ty.span(), &self_ty.to_string());
    let fields = controller.fields();
    let interceptors = &controller.interceptors;

    let build_impl = generate_build_impl(self_ty, &fields);
    let clone_impl = generate_clone_impl(self_ty, &fields);

    #[cfg(feature = "grpc-reflection")]
    let reflection_descriptor_set = quote! {
        Some(include_bytes!(concat!(env!("OUT_DIR"), "/sword_descriptor_set.bin")))
    };

    #[cfg(not(feature = "grpc-reflection"))]
    let reflection_descriptor_set = quote! { None };

    let interceptor_wrappers = interceptors.iter().rev().map(|interceptor| match interceptor {
        InterceptorArgs::Traditional(path) => {
            quote! {
                let interceptor = state.borrow::<#path>().map_err(|err| {
                    #sword_error! {
                        code: "sword.build.grpc",
                        phase: #DiagnosticPhase::Build,
                        target: "gRPC interceptor",
                        title: "Failed to retrieve gRPC interceptor from State",
                        reason: err,
                        context: {
                            "controller" => #stringify!(#self_ty),
                            "interceptor" => #stringify!(#path),
                        },
                        hints: ["Ensure the interceptor is registered and built before gRPC controller setup"],
                        cause: err,
                    }
                })?;

                let service = #tonic_async_interceptor::AsyncInterceptedService::new(
                    service,
                    move |req: #tonic::Request<()>| {
                        let interceptor = #Arc::clone(&interceptor);

                        async move {
                            <#path as #GrpcOnRequest>::on_request(&*interceptor, req).await
                        }
                    },
                );
            }
        }
        InterceptorArgs::WithConfig { interceptor, config } => {
            quote! {
                let interceptor = state.borrow::<#interceptor>().map_err(|err| {
                    #sword_error! {
                        code: "sword.build.grpc",
                        phase: #DiagnosticPhase::Build,
                        target: "gRPC interceptor",
                        title: "Failed to retrieve gRPC interceptor from State",
                        reason: err,
                        context: {
                            "controller" => #stringify!(#self_ty),
                            "interceptor" => #stringify!(#interceptor),
                        },
                        hints: ["Ensure the interceptor is registered and built before gRPC controller setup"],
                        cause: err,
                    }
                })?;

                let service = #tonic_async_interceptor::AsyncInterceptedService::new(
                    service,
                    move |req: #tonic::Request<()>| {
                        let interceptor = #Arc::clone(&interceptor);

                        async move {
                            <#interceptor as #GrpcOnRequestWithConfig<_>>::on_request(
                                &*interceptor,
                                #config,
                                req,
                            ).await
                        }
                    },
                );
            }
        }
        InterceptorArgs::Expression(expr) => {
            quote! {
                let service = #tonic_async_interceptor::AsyncInterceptedService::new(
                    service,
                    #expr,
                );
            }
        }
    });

    let expanded = quote! {
        #build_impl
        #clone_impl

        #inventory::submit! {
            #GrpcControllerRegistrar {
                controller_id: #TypeId::of::<#self_ty>(),
                reflection_descriptor_set: #reflection_descriptor_set,
                build: |state: &#State| -> ::std::result::Result<(), #DiagnosticError> {
                    let controller = #self_ty::build(state).map_err(|e| {
                        #sword_error! {
                            code: "sword.build.grpc",
                            phase: #DiagnosticPhase::Build,
                            target: "gRPC controller",
                            title: "Failed to build gRPC controller",
                            reason: format!("Failed to build the gRPC controller dependency graph: {e}"),
                            source_location: {
                                file: (#location).file(),
                                line: (#location).line(),
                                column: (#location).column(),
                                length: (#location).length(),
                                message: format!("`{}` could not be built", stringify!(#self_ty)),
                            },
                            context: {
                                "controller_name" => #stringify!(#self_ty),
                                "service" => #stringify!(#service),
                            },
                            hints: ["Check the error message for details on what went wrong during construction"],
                            cause: e,
                        }
                    })?;

                    state.insert::<#self_ty>(controller);

                    Ok(())
                },
                register: |_state: &#State, _registry: &mut #GrpcServiceRegistry| -> ::std::result::Result<(), #DiagnosticError> {
                    let state = _state;
                    let registry = _registry;

                    let controller = state.borrow::<#self_ty>().map_err(|err| {
                        #sword_error! {
                            code: "sword.build.grpc",
                            phase: #DiagnosticPhase::Build,
                            target: "gRPC controller",
                            title: "Failed to retrieve gRPC controller from State",
                            reason: err,
                            context: {
                                "controller_name" => #stringify!(#self_ty),
                                "service" => #stringify!(#service),
                            },
                            hints: ["Ensure the controller is built before registration"],
                            cause: err,
                        }
                    })?;

                    let body_limit = state.borrow::<#GrpcBodyLimitValue>().map_err(|err| {
                        #sword_error! {
                            code: "sword.build.grpc",
                            phase: #DiagnosticPhase::Build,
                            target: "gRPC body limit",
                            title: "Failed to retrieve gRPC body-limit settings from State",
                            reason: err,
                            context: {
                                "controller_name" => #stringify!(#self_ty),
                                "service" => #stringify!(#service),
                            },
                            hints: ["Ensure gRPC body-limit settings are inserted into State before controller registration"],
                            cause: err,
                        }
                    })?;

                    let service = <#service<#self_ty>>::new((*controller).clone())
                        .max_decoding_message_size(body_limit.max_decoding_message_size)
                        .max_encoding_message_size(body_limit.max_encoding_message_size);
                    #(#interceptor_wrappers)*
                    registry.routes_builder_mut().add_service(service);
                    registry.mark_service_registered_with_name(<#service<#self_ty> as #tonic::server::NamedService>::NAME);

                    Ok(())
                },
            }
        }

        impl #GrpcController for #self_ty {}

        impl #ControllerSpec for #self_ty {
            fn kind() -> #Controller {
                #Controller::Grpc
            }
        }
    };

    Ok(expanded)
}
