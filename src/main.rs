#![doc = include_str!("../README.md")]

use std::collections::HashMap;

use actix::prelude::*;

mod config;
mod database;
mod gui;
mod mail;

use database::DatabaseActor;
use mail::{FetchMessage, MailActor};

use crate::gui::actor::{GuiActor, StartMessage};

fn main() {
    simple_logger::init().expect("Failed to initialize logging");
    config::init();

    let system = System::new();

    let config = config::GLOBAL_CONFIG
        .get()
        .expect("Configuration has not been initialized");

    let database_addr = system
        .block_on(async { DatabaseActor::start(DatabaseActor::new().await) });

    // Start mail actors for all accounts
    let mut mail_actors: HashMap<String, Addr<MailActor>> = HashMap::new();
    for user in config.get_accounts() {
        let addr = system.block_on(async {
            MailActor::start(MailActor::new(
                user.clone(),
                database_addr.clone(),
            ))
        });
        // This really shouldn't be in the main thread
        addr.do_send(FetchMessage {
            mailbox: ("INBOX".to_owned()),
        });
        mail_actors.insert(user.address.clone(), addr);
    }

    // Start the GUI
    let gui_arbiter = Arbiter::new();
    gui_arbiter.spawn(async move {
        let gui_actor = GuiActor::start(GuiActor::new());
        gui_actor.send(StartMessage).await.expect("GUI actor panicked");
        log::trace!("Go go gadget std::process:exit");
        std::process::exit(0);
    });

    system.run().expect("System aborted");
}
