//! Various tools for handling IMAP functionality

use imap_proto::Address;
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

use crate::config::Account;

/// Represents an email retrieved through IMAP
#[derive(Debug)]
pub(crate) struct ImapEmail {
    /// Headers of the email
    pub(crate) uid: u32,
    /// The envelope of the message
    pub(crate) envelope: Envelope,
}

/// See [RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822#section-3.6) for more details.
#[derive(Debug)]
pub(crate) struct Envelope {
    /// OffsetDateTime parsed by time
    pub(crate) date: Option<OffsetDateTime>,
    /// The subject header
    pub(crate) subject: Option<String>,
    /// The email sender(s)
    pub(crate) _from: Option<Vec<StringAddress>>,
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
// enum ParseHeaderErrors {
//     /// The header data was not valid UTF-8
//     NotUtf8,
// }

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
// fn parse_headers(
//     data: &[u8],
// ) -> Result<HashMap<String, String>, ParseHeaderErrors> {
//     let Ok(header) = std::str::from_utf8(data) else {
//         return Err(ParseHeaderErrors::NotUtf8);
//     };
//     let mut headers_map: HashMap<String, String> = HashMap::new();
//     let mut header = header.chars();
//     let mut buffer = String::new();
//     loop {
//         let Some(character) = header.next() else {
//             break;
//         };
//         if character == '\n' {
//             if let Some(next_character) = header.next() {
//                 if !next_character.is_whitespace() {
//                     let mut split: Vec<String> = buffer
//                         .split(':')
//                         .map(str::trim)
//                         .map(std::string::ToString::to_string)
//                         .collect();
//                     // FIXME: This is pretty reckless. Should probably fix it
//                     // later.
//                     headers_map
//                         .insert(take(&mut split[0]), take(&mut split[1]));
//                     buffer = String::from(next_character);
//                 }
//             }
//             continue;
//         }
//         buffer.push(character);
//     }
//     Ok(headers_map)
// }

/// Process a date into an offset datetime
fn process_date(
    envelope: &imap_proto::types::Envelope,
) -> Option<OffsetDateTime> {
    let date = envelope.date?;
    let string = String::from_utf8(date.to_vec()).ok()?;
    let date = OffsetDateTime::parse(&string, &Rfc2822).ok()?;
    Some(date)
}

/// Turn the subject into a String
fn process_subject(envelope: &imap_proto::types::Envelope) -> Option<String> {
    let subject = envelope.subject?;
    let string = String::from_utf8(subject.to_vec()).ok()?;
    Some(string)
}

/// An email address that contains strings instead of &[u8]
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct StringAddress {
    /// `John Doe` in `John Doe <jdoe@example.com>`
    name: Option<String>,
    /// idk
    adl: Option<String>,
    /// `jdoe` in `John Doe <jdoe@example.com>`
    mailbox: Option<String>,
    /// `example.com` in `John Doe <jdoe@example.com>`
    host: Option<String>,
}

impl StringAddress {
    /// Creates a new string address
    fn new(
        name: Option<String>,
        adl: Option<String>,
        mailbox: Option<String>,
        host: Option<String>,
    ) -> Self {
        Self {
            name,
            adl,
            mailbox,
            host,
        }
    }
}

impl std::fmt::Display for StringAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name.clone().unwrap_or_default();
        let mailbox = self.mailbox.clone().unwrap_or_default();
        let host = self.host.clone().unwrap_or_default();
        write!(f, "{name} <{mailbox}@{host}>")
            .expect("Strings will always format");
        Ok(())
    }
}

/// Process a Vec of addresses
fn process_addresses(
    envelope: &Option<Vec<Address>>,
) -> Option<Vec<StringAddress>> {
    let Some(envelope) = envelope else {
        return None;
    };
    let mut returned = Vec::new();
    for address in envelope {
        let name =
            address.name.and_then(|x| String::from_utf8(x.to_vec()).ok());
        let adl = address.adl.and_then(|x| String::from_utf8(x.to_vec()).ok());
        let mailbox =
            address.mailbox.and_then(|x| String::from_utf8(x.to_vec()).ok());
        let host =
            address.host.and_then(|x| String::from_utf8(x.to_vec()).ok());
        returned.push(StringAddress::new(name, adl, mailbox, host));
    }
    Some(returned)
}

/// Creates an IMAP session with the given server
fn create_session(
    account: &Account,
) -> Result<imap::Session<native_tls::TlsStream<std::net::TcpStream>>, Errors> {
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
    let Ok(imap_session) =
        client.login(&*account.address, &*account.imap_password)
    else {
        return Err(Errors::Login);
    };
    Ok(imap_session)
}

/// Fetch a mailbox for a given account
pub(crate) fn fetch_mailbox(
    account: &Account,
    mailbox: &str,
) -> Result<Vec<ImapEmail>, Errors> {
    let mut imap_session = create_session(account)?;
    if imap_session.select(mailbox).is_err() {
        return Err(Errors::Select);
    };
    let Ok(messages) = imap_session.fetch("*", "(UID ENVELOPE)") else {
        return Err(Errors::Fetch);
    };

    let mut returned: Vec<ImapEmail> = Vec::new();

    for m in &messages {
        let mut date = None;
        let mut subject = None;
        let mut from = None;
        // Process the message envelope
        if let Some(envelope) = m.envelope() {
            let envelope = envelope.to_owned();
            date = process_date(envelope);
            subject = process_subject(envelope);
            from = process_addresses(&envelope.from);
        }

        log::debug!("Date: {date:?}");
        log::debug!("Subject: {subject:?}");
        log::debug!("From: {from:?}");

        returned.push(ImapEmail {
            uid: m.uid.expect("Mail server is not returning UIDs"),
            envelope: Envelope {
                date,
                subject,
                _from: from,
            },
        });
    }
    if imap_session.logout().is_err() {
        return Err(Errors::Logout);
    };
    Ok(returned)
}
