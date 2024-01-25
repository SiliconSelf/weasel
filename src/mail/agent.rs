//! Contains the mail handler thread and its realted structures
//!
//! The star of the show for this crate is `MailAgent`

use std::{thread::JoinHandle, time::Duration};

use crossbeam_channel::{unbounded, Receiver, Sender};
use strum_macros::Display;

use super::imap_toolbox;
use crate::config::{Config, GLOBAL_CONFIG};

/// A struct that owns the thread that handles mail operations.
///
/// As far as the main thread is concerned, this struct is only useful for
/// interactions with `rx`.
pub(crate) struct MailAgent {
    /// The JoinHandle for the thread.
    ///
    /// This really only exists to maintain ownership throughout the lifetime
    /// of the program to keep the borrow checker from killiong the thread
    /// while the program is still in use.
    _handle: JoinHandle<()>,
}

impl MailAgent {
    /// Instantiates a new `MailAgent`, also creating the associated thread.
    pub(crate) fn new(
    ) -> (Sender<MainThreadMessages>, Receiver<MailThreadMessages>, Self) {
        let (main_tx, thread_rx) = unbounded();
        let (thread_tx, main_rx) = unbounded();
        let handle =
            std::thread::spawn(|| mail_agent_thread(thread_tx, thread_rx));
        let agent = Self {
            _handle: handle,
        };
        (main_tx, main_rx, agent)
    }
}

/// Messages that can be sent from the main thread to the `mail_agent_thread`
#[derive(Display)]
pub(crate) enum MainThreadMessages {
    /// Shut down the thread for graceful program exit
    Shutdown,
    /// Reload the configuration to update behavior
    ReloadConfig,
    /// Fetch inboxes from IMAP
    FetchIMAP,
}

/// Messages that can be sent from `mail_agent_thread` to the main thread
#[derive(Display)]
pub(crate) enum MailThreadMessages {
    NewEmail(u32, imap_toolbox::ImapEmail),
    AccountError(String, imap_toolbox::Errors),
}

/// Contains the logic for the mail agent thread
fn mail_agent_thread(
    tx: Sender<MailThreadMessages>,
    rx: Receiver<MainThreadMessages>,
) {
    log::trace!("MailAgent has started");
    let mut config = load_config();
    log::debug!(
        "Loaded accounts: {:?}",
        config
            .get_accounts()
            .iter()
            .map(|a| a.address.clone())
            .collect::<Vec<String>>()
    );
    loop {
        let message = match rx.try_recv() {
            Ok(m) => Some(m),
            Err(crossbeam_channel::TryRecvError::Empty) => {
                std::thread::sleep(Duration::from_millis(250));
                None
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                return;
            }
        };
        if let Some(m) = message {
            log::trace!("Received command {m}");
            match m {
                MainThreadMessages::Shutdown => {
                    return;
                }
                MainThreadMessages::ReloadConfig => {
                    config = load_config();
                }
                MainThreadMessages::FetchIMAP => {
                    for account in config.get_accounts() {
                        match imap_toolbox::fetch_mailbox(account, "INBOX") {
                            Ok(messages) => {
                                for (uid, message) in messages {
                                    tx.send(MailThreadMessages::NewEmail(
                                        uid, message,
                                    ))
                                    .expect(
                                        "Main thread has become disconnected.",
                                    );
                                }
                            }
                            Err(e) => {
                                tx.send(MailThreadMessages::AccountError(
                                    account.address.clone(),
                                    e,
                                ))
                                .expect("Main thread has become disconnected.");
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Clones the inner value of `GLOBAL_CONFIG` and returns it for use in
/// `mail_agent_thread`
fn load_config() -> Config {
    let read_handle = GLOBAL_CONFIG.read();
    let config = read_handle.get().expect("Program is not configured!");
    config.clone()
}
