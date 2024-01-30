//! Database related functionality

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use surrealdb::{engine::local::Mem, sql::Thing, Surreal};

/// Represents an individual retrieved through IMAP
#[derive(Serialize, Deserialize)]
pub(crate) struct Email {
    /// The email headers
    headers: HashMap<String, String>,
}
