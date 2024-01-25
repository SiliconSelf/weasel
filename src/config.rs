//! Holds the global program configuration
//!
//! This module contains the struct of the global program configuration as well
//! as the crate-available lazily-evaluated static value `GLOBAL_CONFIG`, which
//! serves as a thread-safe single source of truth for program configuration.

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Contains the thread-safe global configuration
pub(crate) static GLOBAL_CONFIG: RwLock<OnceCell<Config>> =
    RwLock::new(OnceCell::new());

/// Initialize the `GLOBAL_CONFIG` static
///
/// This configures the program by loading configuration values in the following
/// order. New values overwrite old values.
/// - Default values
/// - `weasel.toml`
/// - Environment variables prefixed with `WEASEL_`
pub(crate) fn init() {
    let config: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file("weasel.toml"))
        .merge(Env::prefixed("WEASEL_"))
        .extract()
        .expect("Failed to load program configuration from environment");
    let write_handle = GLOBAL_CONFIG.write();
    write_handle.set(config).unwrap_or_else(|_| {
        panic!("Failed to set GLOBAL_CONFIG");
    });
    log::debug!("Loaded program configuration");
}

/// Represents a single email account for the `MailAgent` to manage
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Account {
    /// Email address
    pub(crate) address: String,
    /// Domain name to use for SMTP
    pub(crate) smtp_address: String,
    /// Password for the SMTP account
    pub(crate) smtp_password: String,
    /// Port to use for the SMTP server
    pub(crate) smtp_port: u16,
    /// Domain name to use for IMAP
    pub(crate) imap_address: String,
    /// Password to use for the IMAP account
    pub(crate) imap_password: String,
    /// Port to use for the IMAP server
    pub(crate) imap_port: u16,
}

/// Data structure that represents the global program configuration.
/// 
/// Do not derive Debug for this struct. It contains sensitive information!
#[derive(Serialize, Deserialize, Clone, Default)]
pub(crate) struct Config {
    /// Accounts for the MailAgent to manage
    accounts: Vec<Account>,
}

impl Config {
    /// Gets an immutable reference to the accounts in the program configuration
    pub(crate) fn get_accounts(&self) -> &Vec<Account> {
        &self.accounts
    }
}
