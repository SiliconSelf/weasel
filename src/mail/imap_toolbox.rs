//! Various tools for handling IMAP functionality

use std::{collections::HashMap, mem::take, time::SystemTime};

use crate::config::Account;

pub(crate) struct Envelope {
    subject: Option<String>,
    from: Option<Vec<String>>,
    sender: Option<Vec<String>>,
    reply_to: Option<Vec<String>>,
    to: Option<Vec<String>>,
    cc: Option<Vec<String>>,
    bcc: Option<Vec<String>>,
    in_reply_to: Option<Vec<String>>,
}

/// Represents an email retrieved through IMAP
pub(crate) struct ImapEmail {
    /// Headers of the email
    headers: HashMap<String, String>,
    /// Envelope
    envelope: Option<Envelope>,
}

/// Errors that can occur while interacting with IMAP
#[derive(Debug)]
pub(crate) enum Errors {
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
        if character == '\n' {
            if let Some(next_character) = header.next() {
                if !next_character.is_whitespace() {
                    let mut split: Vec<String> = buffer
                        .split(':')
                        .map(str::trim)
                        .map(std::string::ToString::to_string)
                        .collect();
                    // FIXME: This is pretty reckless. Should probably fix it
                    // later.
                    headers_map
                        .insert(take(&mut split[0]), take(&mut split[1]));
                    buffer = String::from(next_character);
                }
            }
            continue;
        }
        buffer.push(character);
    }
    Ok(headers_map)
}

/// Fetch the inbox for a given account
pub(crate) fn fetch_mailbox(
    account: &Account, mailbox: &str,
) -> Result<Vec<(u32, ImapEmail)>, Errors> {
    let tls = native_tls::TlsConnector::builder().build().expect(
        "Failed to build TLS Connector. The application will never work \
         without this.",
    );
    let Ok(client) = imap::connect(
        (&*account.imap_address, account.imap_port),
        &*account.imap_address,
        &tls,
    ) else {
        return Err(Errors::Connect);
    };
    let Ok(mut imap_session) =
        client.login(&*account.address, &*account.imap_password)
    else {
        return Err(Errors::Login);
    };
    if imap_session.select(mailbox).is_err() {
        return Err(Errors::Select);
    };
    let Ok(messages) = imap_session.fetch("*", "(UID ENVELOPE BODY[HEADER])")
    else {
        return Err(Errors::Fetch);
    };

    let mut returned = Vec::new();

    for m in &messages {
        returned.push(m.message);
        match m.header() {
            Some(header) => {
                let Ok(headers) = parse_headers(header) else {
                    panic!("")
                };
                for (k, v) in headers {
                    log::debug!("{k}: {v}");
                }
            }
            None => {
                log::warn!("No body");
            }
        }
    }
    if imap_session.logout().is_err() {
        return Err(Errors::Logout);
    };

    todo!();
}
