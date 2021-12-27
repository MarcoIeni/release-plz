use once_cell::sync::Lazy;
use tracing::Level;
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

pub fn init_test_logs() {
    // Only initialize once logs once
    Lazy::force(&TEST_LOGS);
}

fn _init_test_logs() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    LogTracer::init().expect("Failed to initialise log tracer capturing.");
}

static TEST_LOGS: Lazy<()> = Lazy::new(_init_test_logs);
