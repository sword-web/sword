use std::collections::HashSet;
use tonic::service::{Routes, RoutesBuilder};

#[derive(Default)]
pub struct GrpcServiceRegistry {
    routes: RoutesBuilder,
    service_names: HashSet<&'static str>,
}

impl GrpcServiceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn routes_builder_mut(&mut self) -> &mut RoutesBuilder {
        &mut self.routes
    }

    pub fn mark_service_registered_with_name(&mut self, service_name: &'static str) {
        self.service_names.insert(service_name);
    }

    pub fn services_count(&self) -> usize {
        self.service_names.len()
    }

    pub fn service_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.service_names.iter().copied()
    }

    pub fn into_routes(self) -> Routes {
        self.routes.routes()
    }
}
