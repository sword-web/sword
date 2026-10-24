use crate::diagnostic::{DiagnosticPhase, Error};
use crate::{Build, State, sword_diagnostic};

/// Base trait for all interceptors in Sword.
/// Implement this trait to create interceptors that can be automatically
/// registered and built via the dependency injection system.
///
/// This means that the interceptor can have dependencies injected into it,
/// and also be stored one time and reused  throughout the application lifecycle.
pub trait Interceptor: Build {
    fn register(state: &State) -> Result<(), Error> {
        let interceptor = Self::build(state).map_err(|err| {
            sword_diagnostic! {
                code: "sword.build.interceptor",
                phase: DiagnosticPhase::Build,
                target: "interceptor registration",
                title: "Failed to build interceptor",
                reason: err,
                source: "Interceptor::register",
                hints: ["Ensure interceptor dependencies are registered in the DI container"],
                cause: err,
            }
        })?;

        state.insert(interceptor);

        Ok(())
    }
}

pub struct InterceptorRegistrar {
    pub register: fn(&State) -> Result<(), Error>,
}

inventory::collect!(InterceptorRegistrar);
