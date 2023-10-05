use anyhow::Result;
use clap::Parser;
use config::Configuration;
use firefly::get_transactions_for_account;
use pretty_env_logger::env_logger::Builder;

use args::Arguments;
use log::LevelFilter;

use crate::firefly::history_replay;

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

    for account in &config.accounts {
        println!(
            "Requesting transactions for account: {} ({})",
            account.name, account.firefly_id
        );
        let transactions = get_transactions_for_account(&config, account.firefly_id)?;
        history_replay(&config, account.firefly_id, &account.data, transactions)?;
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
