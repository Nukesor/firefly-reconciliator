//! The main configuration file, that's used to configure this program.
use std::{
    collections::BTreeMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use log::info;
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;

/// All settings which are used by the daemon
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    pub token: String,
    pub accounts: Vec<Account>,
}

/// The historic target data of an account.
/// Entries are years that contain months of target money entries for the end of that month.
/// Money values are in Cent.
/// Structured as such:
/// 2023:
///   1: 512315
///   2: 123410
/// ...
///   12: 190512
pub type BankData = BTreeMap<i32, BTreeMap<u32, i64>>;

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub name: String,
    pub firefly_id: usize,
    pub data: BankData,
}

/// Little helper which expands a given path's `~` characters to a fully qualified path.
pub fn expand_home(old_path: &Path) -> PathBuf {
    PathBuf::from(tilde(&old_path.to_string_lossy()).into_owned())
}

impl Configuration {
    /// Try to read existing config files, while using default values for non-existing fields.
    /// If successful, this will return a full config as well as a boolean on whether we found an
    /// existing configuration file or not.
    ///
    /// The default local config locations depends on the current target.
    pub fn read(from_file: &Option<PathBuf>) -> Result<Configuration> {
        info!("Parsing config files");

        // Load the config from a very specific file path
        let path = if let Some(path) = from_file {
            expand_home(path).clone()
        } else {
            // Get the default path for the user's configuration directory.
            dirs::config_dir()
                .context("Couldn't locate config dir")?
                .join("firefly_reconciliator.yml")
        };

        if !path.exists() || !path.is_file() {
            bail!("Cannot find configuration file at path {path:?}");
        }

        info!("Found config file at: {path:?}");

        // Open the file in read-only mode with buffer.
        let file = File::open(&path).context("Failed to open config file.")?;
        let reader = BufReader::new(file);

        // Read and deserialize the config file.
        let settings =
            serde_yaml::from_reader(reader).context("Failed to deserialize config file")?;

        Ok(settings)
    }
}
