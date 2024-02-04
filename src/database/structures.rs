use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

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
    /// UID
    uid: u32,
    /// Date
    date: Option<OffsetDateTime>,
    /// Subject
    subject: Option<String>,

}

impl From<ImapEmail> for EmailRecord {
    fn from(value: ImapEmail) -> Self {
        Self {
            uid: value.uid,
            date: value.envelope.date,
            subject: value.envelope.subject
        }
    }
}
