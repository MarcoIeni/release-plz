use once_cell::sync::Lazy;
use tracing::Level;
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

pub fn init() {
    // Only initialize once logs once
    Lazy::force(&TEST_LOGS);
}

fn _init() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    LogTracer::init().expect("Failed to initialise log tracer capturing.");
}

static TEST_LOGS: Lazy<()> = Lazy::new(_init);
