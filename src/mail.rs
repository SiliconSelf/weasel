//! Contains the mail handler thread and its realted structures
//!
//! The star of the show for this crate is `MailAgent`

use std::{thread::JoinHandle, time::Duration};

use crossbeam_channel::{unbounded, Receiver, Sender};
use strum_macros::Display;

use crate::config::{Config, GLOBAL_CONFIG};

/// Clones the inner value of `GLOBAL_CONFIG` and returns it for use in
/// `mail_agent_thread`
fn load_config() -> Config {
    let read_handle = GLOBAL_CONFIG.read();
    let config = read_handle.get().expect("Program is not configured!");
    config.clone()
}

/// Contains the logic for the mail agent thread
fn mail_agent_thread(
    tx: Sender<MailAgentMessages>,
    rx: Receiver<MailAgentMessages>,
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
                MailAgentMessages::Shutdown => {
                    return;
                }
                MailAgentMessages::ReloadConfig => {
                    config = load_config();
                }
            }
        }
    }
}

/// Messages that can be sent to and from the mail agent and the main thread
#[derive(Display)]
pub(crate) enum MailAgentMessages {
    /// Shut down the thread for graceful program exit
    Shutdown,
    /// Reload the configuration to update behavior
    ReloadConfig,
}

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
    _handle: JoinHandle<()>,
}

impl MailAgent {
    /// Instantiates a new `MailAgent`, also creating the associated thread.
    pub(crate) fn new() -> Self {
        let (main_tx, thread_rx) = unbounded();
        let (thread_tx, main_rx) = unbounded();
        let handle =
            std::thread::spawn(|| mail_agent_thread(thread_tx, thread_rx));
        Self {
            tx: main_tx,
            rx: main_rx,
            _handle: handle,
        }
    }
    /// Send a message to the mail agent thread. This is the primary function the main thread should interact with the `MailAgent` through.
    pub(crate) fn send_message(&self, message: MailAgentMessages) {
        self.tx.send(message).expect("MailAgent channel has become disconnected");
    }
}
