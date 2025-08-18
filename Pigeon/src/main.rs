mod crypto;
mod error;
#[cfg(feature = "network")]
mod network;
mod storage;
mod ui;
mod config;
mod messaging;
mod identity;

use anyhow::Result;
use clap::Parser;
use ui::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    sodiumoxide::init().expect("sodium init failed");
    // Ensure identity exists for consistent peer ID and crypto keys
    let cfg = crate::config::load();
    let _id = crate::identity::Identity::load_or_generate(&cfg.data_dir)
        .expect("identity load/generate");
    let cli = Cli::parse();
    cli.execute().await?;
    Ok(())
}