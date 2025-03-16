use std::{collections::HashMap, sync::Arc};

use xmpp_parsers::jid::{BareJid, Jid};

//use crate::*;

use tokio_xmpp::{Client, connect::StartTlsServerConnector};

use super::contact::{Contact, ContactId};

use super::id::Id;
use super::{Account, AccountId, Status};

pub type StartTlsClient = Client<StartTlsServerConnector>;

pub struct XmppClientWrapper(pub StartTlsClient);

impl std::fmt::Debug for XmppClientWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "XmppClient<Start Tls Server Connector>")
    }
}

impl XmppClientWrapper {
    #[allow(unused)]
    pub fn into_inner(self) -> StartTlsClient {
        return self.0;
    }
}

impl From<StartTlsClient> for XmppClientWrapper {
    fn from(value: StartTlsClient) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub enum UiEvent {
    Ping,
    RequestContacts(AccountId),
    DisconnectAccount(AccountId),
    NewContact(AccountId, ContactId),
    NewTextMessage {
        from: Jid,
        to: BareJid,
        content: String,
    },
    NewText {
        // TODO replace Arc<BareJid> with a proper struct
        from: Id, // TODO: replace Id with their descriptive counterpart AccountId, and ContactId
        to: Id,
        content: String,
    },
    NewAccount {
        jid: Jid,
        password: String,
    },
    NewXmppClient {
        jid: Jid,
        password: String,
        client: XmppClientWrapper,
    },

    #[deprecated]
    NewXmppClientt(Jid, XmppClientWrapper),
}

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum BackendEvent {
    AccountOnline(AccountId, Jid),
    Account(AccountId, Account),
    AccountStatusUpdate(ContactId, Status),

    Contacts(Vec<(AccountId, ContactId, Contact)>),

    Contact {
        account: AccountId,
        contact: ContactId,
        chats: Vec<(bool, String)>,
    },

    Message {
        to: Id, // TODO: replace Id with AccountId and ContactId
        from: Id,
        body: String,
        by_me: bool,
        timestamp: DateTime<Utc>,
    },
}
