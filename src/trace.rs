//! Tracing

use std::sync::OnceLock;

use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

use crate::config::AppConfig;

/// Static var to indicate that the tracer has been initialized
static INIT_TRACER: OnceLock<()> = OnceLock::new();

/// Initializes the tracer
pub fn init_tracer(cfg: &AppConfig) {
    INIT_TRACER.get_or_init(|| {
        // -> STDOUT
        if cfg.trace.stdout {
            let layer_stdout = tracing_subscriber::fmt::Layer::default();
            let layer_filter = tracing_subscriber::EnvFilter::builder()
                .parse(cfg.trace.filter.as_str())
                .unwrap();

            let trc_subscriber = tracing_subscriber::Registry::default()
                .with(layer_stdout)
                .with(layer_filter);
            tracing::subscriber::set_global_default(trc_subscriber)
                .expect("setting default subscriber failed");
        }
    });
}

#[cfg(test)]
mod tests {
    use tracing::info;

    use super::*;

    #[tracing::instrument]
    async fn do_that() {
        info!("within span");
    }

    #[tokio::test]
    async fn test_tracer() {
        let cfg = crate::config::AppConfig::load().await;
        init_tracer(&cfg);

        info!("INFO before function");
        do_that().await;
        info!("INFO after function");
    }
}
