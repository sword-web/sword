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
path!(type_name => ::std::any::type_name);
path!(Box => ::std::boxed::Box);
path!(Arc => ::std::sync::Arc);
path!(Clone => ::std::clone::Clone);
path!(Debug => ::std::fmt::Debug);
path!(Display => ::std::fmt::Display);
path!(Any => ::std::any::Any);
path!(format => ::std::format);
path!(stringify => ::std::stringify);

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
path!(EventRegistrar => ::sword::internal::events::EventRegistrar);
path!(EventControllerRegistrar => ::sword::internal::events::EventControllerRegistrar);
path!(EventHandlerRegistrar => ::sword::internal::events::EventHandlerRegistrar);
path!(EventHandlerFn => ::sword::internal::events::EventHandlerFn);
path!(EventHandlerResult => ::sword::internal::events::EventHandlerResult);
path!(EventSource => ::sword::events::EventSource);
path!(EventsBoxedFuture => ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::sword::internal::events::EventHandlerResult<()>> + ::std::marker::Send>>);
path!(events_serde_json => ::sword::internal::events::serde_json);
path!(EventCodecError => ::sword::internal::events::EventCodecError);

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
path!(sword_diagnostic => ::sword::internal::core::sword_diagnostic);
path!(DiagnosticPhase => ::sword::internal::core::diagnostic::DiagnosticPhase);
path!(DiagnosticError => ::sword::internal::core::diagnostic::Error);
path!(tracing => ::sword::internal::tracing);

path!(Job => ::sword::internal::jobs::Job);
path!(JobFunctionRegistrar => ::sword::internal::jobs::JobFunctionRegistrar);
path!(JobControllerRegistrar => ::sword::internal::jobs::JobControllerRegistrar);

path!(SocketBoxedFuture => ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()> + ::std::marker::Send>>);

path!(SocketIo => ::sword::socketio::SocketIo);
path!(SocketRef => ::sword::socketio::SocketRef);
path!(SocketContext => ::sword::socketio::SocketContext);
path!(OnConnect => ::sword::socketio::OnConnect);
path!(SocketEventKind => ::sword::internal::socketio::SocketEventKind);
path!(HandlerRegistrar => ::sword::internal::socketio::HandlerRegistrar);
path!(SocketIoHandlerRegistrar => ::sword::internal::socketio::SocketIoHandlerRegistrar);
path!(SocketIoController => ::sword::internal::socketio::SocketIoController);
path!(ConnectHandler => ::sword::internal::socketio::ConnectHandler);

path!(GrpcController => ::sword::internal::grpc::GrpcController);
path!(GrpcControllerRegistrar => ::sword::internal::grpc::GrpcControllerRegistrar);
path!(GrpcServiceRegistry => ::sword::internal::grpc::GrpcServiceRegistry);
path!(GrpcBodyLimitValue => ::sword::internal::grpc::GrpcBodyLimitValue);
path!(tonic => ::sword::internal::grpc::tonic);
path!(tonic_async_interceptor => ::sword::internal::grpc::tonic_async_interceptor);
path!(GrpcStatus => ::sword::grpc::Status);
path!(GrpcCode => ::sword::grpc::Code);
path!(GrpcOnRequest => ::sword::grpc::OnRequest);
path!(GrpcOnRequestWithConfig => ::sword::grpc::OnRequestWithConfig);
