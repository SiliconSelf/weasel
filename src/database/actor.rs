use actix::prelude::*;
use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};

use crate::{database::structures::EmailRecord, mail::ImapEmail};

/// An actor that handles all transactions for a database
pub(crate) struct DatabaseActor {
    /// Connection to the in-memory database
    pub(crate) database: Surreal<Db>,
}

impl DatabaseActor {
    /// Creates a new database
    pub(crate) async fn new() -> Self {
        let db = Surreal::new::<Mem>(())
            .await
            .expect("Failed to create in-memory surreal database");
        Self {
            database: db,
        }
    }
}

impl Actor for DatabaseActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::trace!("Database actor started");
    }
}

/// Message containing a new email to insert into the database
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub(crate) struct NewEmailMessage {
    /// The new email to insert into the database
    pub(crate) email: ImapEmail,
}

impl Handler<NewEmailMessage> for DatabaseActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: NewEmailMessage,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        log::trace!("Database actor received {msg:?}");
        // Create the email record from IMAP response
        let email = msg.email;
        let email_record = EmailRecord::from(email);

        // Run the async database operations
        let database = self.database.clone();
        actix::spawn(async move {
            database.use_ns("weasel").use_db("mail").await.expect(
                "Failed to change to mail database. A malfunctioning database \
                 is not recoverable.",
            );
            let _: Vec<EmailRecord> =
                database.create("mail").content(email_record).await.expect("");
        });
        log::trace!("Added new email");
    }
}

/// Test
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub(crate) struct GuiMessage;

impl Handler<GuiMessage> for DatabaseActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: GuiMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        log::trace!("DatabaseActor received {msg:?}");
    }
}
