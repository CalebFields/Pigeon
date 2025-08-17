mod crypto;
mod error;
#[cfg(feature = "network")]
mod network;
mod storage;
mod ui;

use anyhow::Result;
use clap::Parser;
use ui::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let cli = Cli::parse();
    cli.execute().await?;
    Ok(())
}