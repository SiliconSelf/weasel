//! Contains the mail handler thread and its realted structures
//! 
//! The star of the show for this crate is `MailAgent`

use std::{thread::JoinHandle, time::Duration};

use crossbeam_channel::{unbounded, Receiver};

fn mail_agent_thread() {
    loop {
        std::thread::sleep(Duration::from_millis(250));
    }
}

pub(crate) enum MailAgentMessages {

}

/// A struct that owns the thread that handles mail operations.
/// 
/// As far as the main thread is concerned, this struct is only useful for interactions with `rx`.
pub(crate) struct MailAgent {
    /// The crossbeam_channel receiver for MailAgentMessages from the MailAgent thread
    rx: Receiver<MailAgentMessages>,
    /// The JoinHandle for the thread.
    /// 
    /// This really only exists to maintain ownership throughout the lifetime of the program to keep the borrow checker from killiong the thread while the program is still in use.
    handle: JoinHandle<()>
}

impl MailAgent {
    /// Instantiates a new `MailAgent`, also creating the associated thread.
    pub(crate) fn new() -> Self {
        let (_tx, rx) = unbounded();
        let handle = std::thread::spawn(mail_agent_thread);
        Self {
            rx,
            handle
        }
    }
}