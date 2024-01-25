//! Contains the mail handler thread and its realted structures
//!
//! The star of the show for this crate is `MailAgent`

use std::{thread::JoinHandle, time::Duration};

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::config::GLOBAL_CONFIG;

/// Contains the logic for the mail agent thread
fn mail_agent_thread() {
    log::trace!("MailAgent has started");
    let accounts = {
        let config_handle = GLOBAL_CONFIG.read();
        let config = config_handle.get().expect("Program is not configured!");
        config.get_accounts().clone()
    };
    log::debug!("Loaded accounts to follow:");
    for account in accounts {
        log::debug!("{}", &account.address);
    }
    loop {
        // TODO: Actually do mail shit lmao
        std::thread::sleep(Duration::from_millis(250));
    }
}

/// Messages that can be sent to and from the mail agent and the main thread
pub(crate) enum MailAgentMessages {}

/// A struct that owns the thread that handles mail operations.
///
/// As far as the main thread is concerned, this struct is only useful for
/// interactions with `rx`.
pub(crate) struct MailAgent {
    /// The crossbeam channel transmitter for MailAgentMessages to the
    /// MailAgent thread
    tx: Sender<MailAgentMessages>,
    /// The crossbeam_channel receiver for MailAgentMessages from the MailAgent
    /// thread
    rx: Receiver<MailAgentMessages>,
    /// The JoinHandle for the thread.
    ///
    /// This really only exists to maintain ownership throughout the lifetime
    /// of the program to keep the borrow checker from killiong the thread
    /// while the program is still in use.
    handle: JoinHandle<()>,
}

impl MailAgent {
    /// Instantiates a new `MailAgent`, also creating the associated thread.
    pub(crate) fn new() -> Self {
        let (tx, rx) = unbounded();
        let handle = std::thread::spawn(mail_agent_thread);
        Self {
            tx,
            rx,
            handle,
        }
    }
}
