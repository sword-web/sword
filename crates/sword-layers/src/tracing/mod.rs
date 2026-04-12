mod config;
mod dev;
mod time;

use dev::DevFormatter;
use std::{io, sync::OnceLock};
use time::TimestampFormatter;
use tracing_subscriber::{
    EnvFilter, Registry,
    layer::{Layer, SubscriberExt},
};

pub use config::*;

type BoxLayer = Box<dyn Layer<Registry> + Send + Sync + 'static>;
static TRACING_INIT: OnceLock<()> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct TracingSubscriber {
    config: TracingConfig,
}

impl TracingSubscriber {
    pub fn env_filter(&self) -> EnvFilter {
        if self.config.use_env_filter {
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new(self.config.filter.clone()))
                .unwrap_or_else(|_| EnvFilter::new("info"))
        } else {
            EnvFilter::try_new(self.config.filter.clone())
                .unwrap_or_else(|_| EnvFilter::new("info"))
        }
    }

    pub fn fmt_layer(&self) -> BoxLayer {
        let config = &self.config;
        let timer = TimestampFormatter::from_config(config);

        let builder = tracing_subscriber::fmt::layer()
            .with_ansi(config.format != LogFormat::Json)
            .with_thread_ids(config.has_field(TracingField::ThreadId))
            .with_thread_names(config.has_field(TracingField::ThreadName))
            .with_writer(io::stdout)
            .with_timer(timer.clone())
            .with_target(config.has_field(TracingField::Target))
            .with_file(config.has_field(TracingField::File))
            .with_line_number(config.has_field(TracingField::LineNumber));

        match config.format {
            LogFormat::Full => Box::new(builder),
            LogFormat::Pretty => Box::new(builder.pretty()),
            LogFormat::Compact => Box::new(builder.compact()),
            LogFormat::Json => Box::new(builder.json().flatten_event(true)),

            LogFormat::Dev => {
                let dev_fmt = DevFormatter::new(config);
                let builder = builder.event_format(dev_fmt);

                Box::new(builder)
            }
        }
    }

    pub fn layer(&self) -> BoxLayer {
        Box::new(self.fmt_layer().with_filter(self.env_filter()))
    }

    pub fn init(self) -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
        if !self.config.enabled {
            return Ok(());
        }

        if TRACING_INIT.get().is_some() || tracing::dispatcher::has_been_set() {
            return Ok(());
        }

        let config = self.config.clone();
        let subscriber = Registry::default().with(self.layer());

        match tracing::subscriber::set_global_default(subscriber) {
            Ok(()) => {
                let _ = TRACING_INIT.set(());

                tracing::info!(
                    target: "sword.startup.tracing",
                    format = ?config.format,
                    default_filter = %config.filter,
                    use_env_filter = config.use_env_filter,
                    "Initialized tracing subscriber"
                );
            }
            Err(_) if tracing::dispatcher::has_been_set() => {
                let _ = TRACING_INIT.set(());
            }
            Err(err) => return Err(err),
        }

        Ok(())
    }
}

impl From<TracingConfig> for TracingSubscriber {
    fn from(config: TracingConfig) -> Self {
        Self { config }
    }
}
