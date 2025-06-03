use anyhow::Result;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use tracing_appender::{non_blocking, rolling};

pub fn init_logging(log_level: &str) -> Result<()> {
    // Create log directory if it doesn't exist
    std::fs::create_dir_all("logs")?;

    // File appender with daily rotation
    let file_appender = rolling::daily("logs", "sniper_bot.log");
    let (non_blocking_file, _guard) = non_blocking(file_appender);

    // Console output
    let (non_blocking_stdout, _guard2) = non_blocking(std::io::stdout());

    // Environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    // Initialize subscriber with both file and console output
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::Layer::new()
                .with_writer(non_blocking_stdout)
                .with_ansi(true)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
        )
        .with(
            fmt::Layer::new()
                .with_writer(non_blocking_file)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .json()
        )
        .init();

    Ok(())
}
