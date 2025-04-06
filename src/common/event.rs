use tokio_xmpp::parsers::jid::{BareJid, Jid};
use tokio_xmpp::{Client, connect::StartTlsServerConnector};
use super::{Account, AccountId, Status, Id, contact::{Contact, ContactId}};


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
    // Ping,
    // RequestContacts(AccountId),
    DisconnectAccount(AccountId),
    NewContact(AccountId, ContactId),
    NewTextMessage {
        from: Jid,
        to: BareJid,
        content: String,
    },
    NewText {
        // TODO replace Arc<BareJid> with a proper struct
        from: AccountId, // TODO: replace Id with their descriptive counterpart AccountId, and ContactId
        to: ContactId,
        content: String,
    },
    NewXmppClient {
        jid: Jid,
        password: String,
        client: XmppClientWrapper,
    },
}

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum BackendEvent {
    Account(AccountId, Account),
    AccountStatusUpdate(AccountId, Status),

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
        id: Option<String>,
    },
}
