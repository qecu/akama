use std::ops::Deref;
use chrono::NaiveDateTime;
use std::sync::Arc;

use xmpp_parsers::jid::Jid;
use xmpp_parsers::jid::BareJid;
use super::Message;
use super::Status;

use super::id::Id;

pub type ContactId = Id;

//#[derive(Debug, Clone, PartialEq, Eq, Hash)]
//pub struct ContactIdd(Arc<BareJid>);
//
//unsafe impl Send for ContactIdd {}
//
//unsafe impl Sync for ContactIdd {}
//
//impl AsRef<BareJid> for ContactIdd {
//    fn as_ref(&self) -> &BareJid {
//        self.deref() 
//    }
//}
//
//impl Deref for ContactIdd {
//    type Target = BareJid;
//
//    fn deref(&self) -> &Self::Target {
//        self.0.deref()
//    }
//}



#[derive(Debug, Clone)]
pub struct Contact {
    jid: ContactId,
    status: Status,
    pub chat_history: Vec<Message>
}

impl Contact {
    pub fn new(
        jid: ContactId, 
        status: Status,
        chat_history: Vec<Message>,
    ) -> Self {
        Self {
            jid,
            status,
            chat_history,
        }
    }

    pub fn jid(&self) -> &Jid {
        &self.jid
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn clear_chat_history(&mut self) {
        self.chat_history.clear()
    }

    pub fn new_text(&mut self, text: String, by_me: bool) {
        let naive = chrono::Utc::now().naive_utc();
        self.chat_history.push(Message::new_text(text, by_me, naive));
    }
}
