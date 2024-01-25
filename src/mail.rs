//! Contains the mail handler thread and its realted structures
//!
//! The star of the show for this crate is `MailAgent`

use std::{
    collections::HashMap, mem::take, thread::JoinHandle, time::Duration,
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::config::{Account, Config, GLOBAL_CONFIG};

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
    NewEmail(ImapEmail),
}

/// Represents an email retrieved through IMAP
#[derive(Serialize, Deserialize)]
pub(crate) struct ImapEmail {}

/// Contains the logic for the mail agent thread
fn mail_agent_thread(
    tx: Sender<MailThreadMessages>,
    rx: Receiver<MainThreadMessages>,
) {
    log::trace!("MailAgent has started");
    let mut config = load_config();
    let mut messages: HashMap<u32, ImapEmail> = HashMap::new();
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
                        fetch_mailbox(account, "INBOX").unwrap();
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

/// Errors that can occur while interacting with IMAP
#[derive(Debug)]
pub(crate) enum ImapErrors {
    /// The TLS connector cannot connect to the given inbox. A wrong domain or
    /// port was probably given.
    Connect,
    /// The IMAP client can't login to the server. The account is probably
    /// misconfigured with a wrong username or password.
    Login,
    /// The client can't select the given inbox
    Select,
    /// The client can't fetch messages from the given inbox
    Fetch,
    /// The client failed to logout. I'm honestly not sure how this would
    /// happen, but it can.
    Logout,
}

/// Fetch the inbox for a given account
fn fetch_mailbox(
    account: &Account,
    mailbox: &str,
) -> Result<Vec<(u32, ImapEmail)>, ImapErrors> {
    let tls = native_tls::TlsConnector::builder().build().expect(
        "Failed to build TLS Connector. The application will never work \
         without this.",
    );
    let Ok(client) = imap::connect(
        (&*account.imap_address, account.imap_port),
        &*account.imap_address,
        &tls,
    ) else {
        return Err(ImapErrors::Connect);
    };
    let Ok(mut imap_session) =
        client.login(&*account.address, &*account.imap_password)
    else {
        return Err(ImapErrors::Login);
    };
    if imap_session.select(mailbox).is_err() {
        return Err(ImapErrors::Select);
    };
    // let Ok(messages) = imap_session.fetch("* (FLAGS BODY[HEADER.FIELDS (DATE
    // FROM)])", "RFC822") else {     return Err(ImapErrors::Fetch);
    // };
    let messages = imap_session.fetch("*", "BODY[HEADER]").unwrap();

    let mut returned = Vec::new();

    for m in &messages {
        returned.push(m.message);
        if let Some(header) = m.header() {
            let Ok(headers) = parse_headers(header) else {
                panic!("")
            };
        } else {
            log::warn!("No body");
        }
    }
    if imap_session.logout().is_err() {
        return Err(ImapErrors::Logout);
    };

    todo!();
}

/// Errors that can occur while parsing email headers
enum ParseHeaderErrors {
    /// The header data was not valid UTF-8
    NotUtf8,
}

/// Parse a binary representation of message headers to make them something
/// useful instead of the format the braindead lobotomites cooked up in the
/// early 2000s
///
/// Message headers are separated by newline characters, but header values can
/// contain newline characters as long as they're before any whitespace
/// character.
///
/// This function does some pretty heinous memory allocations and is probably
/// far more expensive than it has any right to be because of it. It should
/// probably be rewritten at some point in the future because parsing email
/// headers is a pretty common task for an email client.
fn parse_headers(
    data: &[u8],
) -> Result<HashMap<String, String>, ParseHeaderErrors> {
    let Ok(header) = std::str::from_utf8(data) else {
        return Err(ParseHeaderErrors::NotUtf8);
    };
    let mut headers_map: HashMap<String, String> = HashMap::new();
    let mut header = header.chars();
    let mut buffer = String::new();
    loop {
        let Some(character) = header.next() else {
            break;
        };
        buffer.push(character);
        if character == '\n' {
            if let Some(next_character) = header.next() {
                if !next_character.is_whitespace() {
                    let mut split: Vec<String> = buffer
                        .split(':')
                        .map(str::trim)
                        .map(std::string::ToString::to_string)
                        .collect();
                    log::debug!("{split:?}");
                    // FIXME: This is pretty reckless. Should probably fix it
                    // later.
                    headers_map
                        .insert(take(&mut split[0]), take(&mut split[1]));
                    buffer = String::from(next_character);
                }
            }
        }
    }
    Ok(headers_map)
}
