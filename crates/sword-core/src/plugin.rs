use crate::{ComponentRegistry, Config, ProviderRegistry};

#[allow(async_fn_in_trait)]
#[allow(unused_variables)]
pub trait Plugin {
    fn name() -> &'static str;

    /// Register component structs marked with `#[injectable]`
    fn register_components(components: &ComponentRegistry) {}

    /// Register provider structs marked with `#[injectable(provider)]`
    async fn register_providers(config: &Config, providers: &ProviderRegistry) {}
}
