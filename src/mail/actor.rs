//! Contains the mail handler thread and its realted structures
//!
//! The star of the show for this crate is `MailAgent`

use actix::prelude::*;

use super::imap_toolbox;
use crate::config::Account;
/// An actor that handles all transactions for a given email account
pub(crate) struct MailActor {
    /// The address this actor represents
    pub(crate) account: Account,
}

impl MailActor {
    /// Creates a new actor for a given account
    pub(crate) fn new(account: &Account) -> Self {
        Self { account: account.clone() }
    }
}

impl Actor for MailActor {
    type Context = Context<Self>;
}

/// A Message to fetch a given inbox from the account this actor represents
#[derive(Message, Debug)]
#[rtype(result = "Result<Vec<(u32, imap_toolbox::ImapEmail)>, \
                  imap_toolbox::Errors>")]
pub(crate) struct FetchMessage {
    /// Which mailbox to fetch
    mailbox: String,
}

impl Handler<FetchMessage> for MailActor {
    type Result =
        Result<Vec<(u32, imap_toolbox::ImapEmail)>, imap_toolbox::Errors>;

    fn handle(
        &mut self, msg: FetchMessage, _ctx: &mut Context<Self>,
    ) -> Self::Result {
        log::trace!("Actor for {} received {msg:?}", self.account.address);
        match imap_toolbox::fetch_mailbox(&self.account, &msg.mailbox) {
            Ok(emails) => { Ok(emails) },
            Err(e) => Err(e)
        }
    }
}

