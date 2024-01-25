//! Contains the mail handler thread and its realted structures
//!
//! The star of the show for this crate is `MailAgent`

use std::{thread::JoinHandle, time::Duration};

use crossbeam_channel::{unbounded, Receiver, Sender};
use strum_macros::Display;

use crate::config::{Account, Config, GLOBAL_CONFIG};

use serde::{Serialize, Deserialize};

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
    pub(crate) fn new() -> (Sender<MainThreadMessages>, Receiver<MailThreadMessages>, Self) {
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
    FetchIMAP
}

/// Messages that can be sent from `mail_agent_thread` to the main thread
#[derive(Display)]
pub(crate) enum MailThreadMessages {
    NewEmail(IMAPEmail)
}

/// Represents an email retrieved through IMAP
#[derive(Serialize, Deserialize)]
pub(crate) struct IMAPEmail {

}

/// Contains the logic for the mail agent thread
fn mail_agent_thread(
    tx: Sender<MailThreadMessages>,
    rx: Receiver<MainThreadMessages>,
) {
    log::trace!("MailAgent has started");
    let mut config = load_config();
    log::debug!("Loaded accounts to follow:");
    for account in config.get_accounts() {
        log::debug!("{}", &account.address);
    }
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
                        fetch_inbox(account);
                        // let messages = fetch_inbox(account);
                        // for message in messages {
                        //     tx.send(MailThreadMessages::NewEmail(message));
                        // }
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

/// Fetch the inbox for a given account
/// 
/// This function is currently wild unsafe and uses unwrap() on almost every line. It needs to be rewritten,
fn fetch_inbox(account: &Account) {
    let tls = native_tls::TlsConnector::builder().build().unwrap();
    let client = imap::connect((account.imap_address.clone(), account.imap_port), account.imap_address.clone(), &tls).unwrap();
    let mut imap_session = client.login(account.address.clone(), account.imap_password.clone()).unwrap();
    imap_session.select("INBOX").unwrap();
    let messages = imap_session.fetch("1", "RFC822").unwrap();
    if let Some(m) = messages.iter().next() {
        let body = m.body().unwrap();
        let body = std::str::from_utf8(body).unwrap().to_owned();
        log::info!("{body}");
    }
    imap_session.logout().unwrap();
}