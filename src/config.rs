//! Holds the global program configuration
//!
//! This module contains the struct of the global program configuration as well
//! as the crate-available lazily-evaluated static value GLOBAL_CONFIG, which
//! serves as a thread-safe single source of truth for program configuration.

use once_cell::sync::Lazy;
use serde::Deserialize;

pub(crate) static GLOBAL_CONFIG: Lazy<Config> = Lazy::new(|| Config {});

#[derive(Deserialize)]
pub(crate) struct Config {}
