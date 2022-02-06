use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

/// Use `info` level by default, but you can customize it with `RUST_LOG` environment variable.
pub fn init() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .with_line_number(true)
        .pretty()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed.");
    LogTracer::init().expect("Failed to initialise log tracer capturing.");
}
