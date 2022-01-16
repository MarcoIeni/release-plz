use tracing::Level;
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

pub fn init() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed.");
    LogTracer::init().expect("Failed to initialise log tracer capturing.");
}
