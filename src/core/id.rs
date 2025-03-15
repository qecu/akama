use std::{ops::Deref, sync::Arc};
use tokio_xmpp::jid::{BareJid, Jid};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// TODO: perhaps make the inner private in the future
// TODO: find a more descritpive name if poosible
pub struct Id(pub Arc<BareJid>);

impl From<Jid> for Id {
    fn from(value: Jid) -> Self {
        Id(Arc::new(value.to_bare()))
    }
}

impl From<BareJid> for Id {
    fn from(value: BareJid) -> Self {
        Id(Arc::new(value))
    }
}

impl AsRef<BareJid> for Id {
    fn as_ref(&self) -> &BareJid {
        self.deref() 
    }
}

impl Deref for Id {
    type Target = BareJid;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

unsafe impl Send for Id {}

unsafe impl Sync for Id {}

