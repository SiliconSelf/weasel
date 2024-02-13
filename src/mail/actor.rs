//! Contains the mail handler thread and its realted structures
//!
//! The star of the show for this crate is `MailAgent`

use actix::prelude::*;

use super::imap_toolbox::{self, Errors};
use crate::{config::Account, database::DatabaseActor};

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
        account: Account,
        db_address: Addr<DatabaseActor>,
    ) -> Self {
        Self {
            account,
            db_address,
        }
    }
}

impl Actor for MailActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::trace!("Started mail actor for {}", self.account.address);
    }
}

/// A Message to fetch a given inbox from the account this actor represents
#[derive(Message, Debug)]
#[rtype(result = "Result<(), Errors>")]
pub(crate) struct FetchMessage {
    /// Which mailbox to fetch
    pub(crate) mailbox: String,
}

impl Handler<FetchMessage> for MailActor {
    type Result = Result<(), Errors>;

    fn handle(
        &mut self,
        msg: FetchMessage,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        log::trace!("Actor for {} received {msg:?}", self.account.address);
        let mail =
            match imap_toolbox::fetch_mailbox(&self.account, &msg.mailbox) {
                Ok(mail) => mail,
                Err(e) => {
                    log::warn!(
                        "Actor for {} received error \"{e:?}\" when running \
                         {msg:?}",
                        self.account.address
                    );
                    return Err(e);
                }
            };
        log::trace!("Actor for {} fetched mail", self.account.address);
        for message in mail {
            let address = self.db_address.clone();
            actix::spawn(async move {
                log::debug!("Sending message to database");
                let _weh = address
                    .send(crate::database::NewEmailMessage {
                        email: message,
                    })
                    .await
                    .expect("Sending message failed");
                log::debug!("Message sent to database");
            });
        }
        Ok(())
    }
}
