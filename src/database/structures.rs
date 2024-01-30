use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::mail::ImapEmail;

/// Represents a contact from the emails
#[derive(Serialize, Deserialize)]
pub(crate) struct ContactRecord {
    /// The person's email address
    address: String,
}

/// Represents an individual retrieved through IMAP
#[derive(Serialize, Deserialize)]
pub(crate) struct EmailRecord {
    /// The email headers
    headers: HashMap<String, String>,
}

impl From<ImapEmail> for EmailRecord {
    fn from(value: ImapEmail) -> Self {
        Self {
            headers: value.headers
        }
    }
}