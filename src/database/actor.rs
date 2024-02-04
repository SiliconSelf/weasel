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
        log::debug!("Created database actor");
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
#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub(crate) struct NewEmailMessage {
    /// The new email to insert into the database
    pub(crate) email: ImapEmail,
}

impl Handler<NewEmailMessage> for DatabaseActor {
    type Result = Result<(), ()>;

    fn handle(
        &mut self,
        msg: NewEmailMessage,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        log::trace!(
            "Database actor received new email to insert into database"
        );
        // Create the email record from IMAP response
        let email = msg.email;
        let email_record = EmailRecord::from(email);

        // Run the async database operations
        let database = &self.database;
        futures::executor::block_on(async {
            database.use_db("mail").await.expect(
                "Failed to change to mail database. A malfunctioning database \
                 is not recoverable.",
            );
            let _: Vec<EmailRecord> =
                database.create("mail").content(email_record).await.expect("");
        });

        Ok(())
    }
}
