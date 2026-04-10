use axum::{Router, extract::Request, response::IntoResponse, routing::Route};
use std::convert::Infallible;
use tower::{Layer, Service};

type LayerFn<S> = Box<dyn Fn(Router<S>) -> Router<S> + Send + Sync>;

/// A stack for managing and applying middleware layers to a router.
///
/// `LayerStack` provides a way to accumulate layers and apply them to a router in the
/// order they were added. Layers are applied via the `push()` method during configuration,
/// and then applied to the router via `apply()` during the build phase.
pub struct LayerStack<S> {
    layers: Vec<LayerFn<S>>,
}

impl<S> LayerStack<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Add a layer to the stack.
    ///
    /// Layers are applied in FIFO order when `apply()` is called. Each layer will
    /// wrap the router after all controllers have been registered.
    pub fn push<L>(&mut self, layer: L)
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        self.layers.push(Box::new(move |router: Router<S>| {
            router.layer(layer.clone())
        }));
    }

    /// Apply all layers to the given router.
    ///
    /// Layers are applied in FIFO order (first pushed = first applied = outermost).
    pub fn apply(self, mut router: Router<S>) -> Router<S> {
        for layer_fn in self.layers {
            router = layer_fn(router);
        }
        router
    }
}

impl<S> Default for LayerStack<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
