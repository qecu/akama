use tokio_xmpp::jid::Jid;
use super::{Message, Status, Id};


pub type ContactId = Id;

#[derive(Debug, Clone)]
pub struct Contact {
    jid: ContactId,
    status: Status,
    pub chat_history: Vec<Message>,
}

impl Contact {
    pub fn new(jid: ContactId, status: Status, chat_history: Vec<Message>) -> Self {
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
        self.chat_history
            .push(Message::new_text(text, by_me, naive));
    }
}
