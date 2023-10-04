use anyhow::Result;
use clap::Parser;
use config::Configuration;
use firefly::get_transactions_for_account;
use pretty_env_logger::env_logger::Builder;

use args::Arguments;
use log::LevelFilter;
use reqwest::{
    blocking::ClientBuilder,
    header::{HeaderMap, HeaderValue},
};

mod args;
mod config;
mod firefly;

fn main() -> Result<()> {
    // Read any .env files
    dotenv::dotenv().ok();
    // Parse commandline options.
    let args = Arguments::parse();

    // Initalize everything
    init_app(args.verbose)?;

    let config = Configuration::read(&args.config)?;

    // Prepare default headers for firefly api.
    let mut headers = HeaderMap::new();
    headers.insert("accept", HeaderValue::from_str("application/vnd.api+json")?);
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", config.token))?,
    );
    headers.insert("Content-Type", HeaderValue::from_str("application/json")?);

    let client = ClientBuilder::new().default_headers(headers).build()?;

    for account in config.accounts {
        let data = get_transactions_for_account(&client, account.firefly_id)?;
        println!("{data:#?}");
    }

    Ok(())
}

/// Init better_panics
/// Initialize logging
fn init_app(verbosity: u8) -> Result<()> {
    // Beautify panics for better debug output.
    better_panic::install();

    // Set the verbosity level and initialize the logger.
    let level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    Builder::new().filter_level(level).init();

    Ok(())
}
