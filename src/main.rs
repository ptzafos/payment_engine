use std::{fs::File, path::PathBuf};

use clap::Parser;
use eyre::Result;
use payments_engine::PaymentEngine;
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(value_name = "TRANSACTIONS_CSV")]
    input: PathBuf,
}

#[tokio::main]
// customize errors reminder
async fn main() -> Result<()> {
    let args = Cli::parse();

    // fmt()
    //     .with_env_filter(
    //         EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
    //     )
    //     .init();

    let file = File::open(&args.input)?;
    PaymentEngine::start_app(file).await?;
    Ok(())
}
