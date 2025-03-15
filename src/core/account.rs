use std::{ops::Deref, sync::Arc};

use tokio_xmpp::jid::BareJid;
use xmpp_parsers::jid::Jid;
use super::Status;

use super::id::Id;
pub type AccountId = Id;


/// just a type wrapper for better readiblity (hopefuly)
//pub type AccountId = BareJid;

// TODO: make the member private
//#[derive(Debug, Clone, PartialEq, Eq, Hash)]
//pub struct AccountId(pub Arc<BareJid>);
//
//unsafe impl Send for AccountId {}
//
//unsafe impl Sync for AccountId {}
//
//impl AsRef<BareJid> for AccountId {
//    fn as_ref(&self) -> &BareJid {
//        self.deref() 
//    }
//}
//
//impl Deref for AccountId {
//    type Target = BareJid;
//
//    fn deref(&self) -> &Self::Target {
//        self.0.deref()
//    }
//}
//
//impl std::fmt::Display for AccountId {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        write!(f, "{}", self.0)
//    }
//}


#[derive(Debug, Clone)]
pub struct Account {
    jid: Jid,
    status: Status, 
}

impl Account {
    pub fn new(jid: Jid, status: Status) -> Self {
        Self {
            jid,
            status,
        }
    }

    pub fn set_jid(&mut self, jid: Jid) {
        self.jid = jid;
    }

    pub fn jid(&self) -> &Jid {
        &self.jid
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status
    }

}
