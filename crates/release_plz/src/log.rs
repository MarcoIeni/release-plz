use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

/// Use `info` level by default, but you can customize it with `RUST_LOG` environment variable.
pub fn init(verbose: bool) {
    let subscriber = {
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
        let mut event_fmt = tracing_subscriber::fmt::format().pretty();
        if !verbose {
            event_fmt = event_fmt.with_target(false).with_source_location(false);
        }
        FmtSubscriber::builder()
            .with_env_filter(env_filter)
            .with_writer(std::io::stderr)
            .event_format(event_fmt)
            .finish()
    };

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed.");
    LogTracer::init().expect("Failed to initialise log tracer capturing.");
}
