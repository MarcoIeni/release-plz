use std::io;

use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

/// Use `info` level by default, but you can customize it with `RUST_LOG` environment variable.
pub fn init() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let layer = tracing_tree::HierarchicalLayer::default()
        .with_writer(io::stderr)
        .with_indent_lines(true)
        .with_ansi(true)
        .with_targets(true)
        .with_indent_amount(2);
    let subscriber = tracing_subscriber::Registry::default()
        .with(layer)
        .with(env_filter);
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed.");
    LogTracer::init().expect("Failed to initialise log tracer capturing.");
}
