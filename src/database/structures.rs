use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Represents a contact from the emails
#[derive(Serialize, Deserialize)]
pub(crate) struct Contact {
    /// The person's email address
    address: String,
}

/// Represents an individual retrieved through IMAP
#[derive(Serialize, Deserialize)]
pub(crate) struct Email {
    /// The email headers
    headers: HashMap<String, String>,
}
