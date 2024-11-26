use anyhow::{bail, Result};
use std::str::FromStr;
use tracing_subscriber::{filter, fmt, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let config = servy::Config::from_env()?;

    // Initialise logging with the log level from the config
    tracing_subscriber::registry()
        .with(filter::LevelFilter::from_str(&config.log_level)?)
        .with(fmt::layer().json().with_target(false))
        .init();

    if let Err(e) = servy::run(config).await {
        bail!("error occurred in servy: {}", e)
    }

    Ok(())
}
