//! Tracing

use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

use crate::config::AppConfig;

/// Initializes the tracer
pub fn init_tracer(cfg: &AppConfig) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tracing::instrument]
    async fn do_that() {
        tracing::info!("within span");
    }

    #[tokio::test]
    async fn test_init() {
        let cfg = crate::config::AppConfig::load().await;

        init_tracer(cfg);

        tracing::info!("INFO before function");
        do_that().await;
        tracing::info!("INFO after function");
    }
}
