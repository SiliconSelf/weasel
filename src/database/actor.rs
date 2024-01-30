use actix::{Actor, Context};
use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};

/// An actor that handles all transactions for a database
pub(crate) struct DatabaseActor {
    /// Connection to the in-memory database
    pub(crate) _database: Surreal<Db>,
}

impl DatabaseActor {
    /// Creates a new database
    pub(crate) async fn new() -> Self {
        let db = Surreal::new::<Mem>(())
            .await
            .expect("Failed to create in-memory surreal database");
        Self {
            _database: db,
        }
    }
}

impl Actor for DatabaseActor {
    type Context = Context<Self>;
}
