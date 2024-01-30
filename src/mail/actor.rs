//! Contains the mail handler thread and its realted structures
//!
//! The star of the show for this crate is `MailAgent`

use actix::prelude::*;

use super::imap_toolbox::{self, Errors};
use crate::{config::Account, database::{DatabaseActor, NewEmailMessage}};

/// An actor that handles all transactions for a given email account
pub(crate) struct MailActor {
    /// The address this actor represents
    pub(crate) account: Account,
    /// Address of the database actor for inter-actor communication
    db_address: Addr<DatabaseActor>,
}

impl MailActor {
    /// Creates a new actor for a given account
    pub(crate) fn new(
        account: &Account, db_address: Addr<DatabaseActor>,
    ) -> Self {
        Self {
            account: account.clone(),
            db_address,
        }
    }
}

impl Actor for MailActor {
    type Context = Context<Self>;
}

/// A Message to fetch a given inbox from the account this actor represents
#[derive(Message, Debug)]
#[rtype(result = "Result<(), Errors>")]
pub(crate) struct FetchMessage {
    /// Which mailbox to fetch
    mailbox: String,
}

impl Handler<FetchMessage> for MailActor {
    type Result = Result<(), Errors>;

    fn handle(
        &mut self, msg: FetchMessage, _ctx: &mut Context<Self>,
    ) -> Self::Result {
        log::trace!("Actor for {} received {msg:?}", self.account.address);
        let mail =
            match imap_toolbox::fetch_mailbox(&self.account, &msg.mailbox) {
                Ok(mail) => mail,
                Err(e) => {
                    return Err(e);
                }
            };
            for (_, message) in mail {
                self.db_address.do_send(NewEmailMessage { email: message });
            }
        Ok(())
    }
}
