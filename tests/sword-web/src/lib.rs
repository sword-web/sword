#[cfg(test)]
pub fn application_builder() -> sword::ApplicationBuilder {
    sword::Application::from_config_path("Config.toml")
}

#[cfg(test)]
pub fn test_server(app: sword::Application) -> axum_test::TestServer {
    axum_test::TestServer::new(app.router()).unwrap()
}

#[cfg(test)]
mod request {
    mod cookies;
    mod multipart;
    mod query;
    mod stream;
}

#[cfg(test)]
mod errors;

#[cfg(test)]
mod http_methods;

#[cfg(test)]
mod interceptors {
    mod built_in;
    mod controller_level;
    mod handler_level;
}

#[cfg(test)]
mod application {
    mod config;
    mod di;
}

#[cfg(test)]
pub mod utils;
