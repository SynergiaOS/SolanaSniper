use anyhow::Result;
use tracing_subscriber::EnvFilter;

pub fn init_logging(log_level: &str) -> Result<()> {
    // Create log directory if it doesn't exist
    std::fs::create_dir_all("logs")?;

    // Environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    // Simple console and file logging without non-blocking writers
    // to avoid guard lifetime issues
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_ansi(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    println!("🔧 Logging initialized with level: {}", log_level);
    Ok(())
}
