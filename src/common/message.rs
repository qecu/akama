use chrono::NaiveDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub content: Content,
    pub by_me: bool,
    pub stamp: NaiveDateTime,
    //pub from: Jid,
    //pub to: Jid,
}

impl Message {
    pub fn new_text(text: String, by_me: bool, stamp: NaiveDateTime) -> Self {
        Self {
            content: Content::Text(text),
            by_me,
            stamp, //from,
                   //to
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    Text(String),
}
