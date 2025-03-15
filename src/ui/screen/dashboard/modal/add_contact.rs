use std::sync::Arc;

use async_channel::Sender;
use crate::core::event::UiEvent;
use iced::widget::{self, *, column};
use iced::*;
use tokio_xmpp::jid::BareJid;
use super::modal;
use crate::core::contact::ContactId;


#[derive(Debug, Clone)]
pub enum Message {
    Open,
    Close,
    Submit,
    TextInputJid(String),
}

pub struct AddContact {
    pub show: bool, 
    pub sender: Sender<UiEvent>,
    pub text_input_jid: String,
    pub error: String,
}

impl AddContact {

    pub fn new(sender: Sender<UiEvent>) -> Self {
        Self {
            sender,
            show: false,
            text_input_jid: String::new(),
            error: String::new(),
        }
    }

    // TODO find a more descriptive name 
    pub fn update<F>(
        &mut self, 
        message: Message,
        mut new_contact: F        
    ) -> Task<Message> 
    where
        F: FnMut(ContactId)
    {
        match message {
            Message::Open => { 
                self.show = true;
                return widget::focus_next();
            },
            Message::Close => self.close(),
            Message::TextInputJid(s) => self.text_input_jid = s,
            Message::Submit => {
                if let Ok(jid) = BareJid::new(&self.text_input_jid) {
                    new_contact(jid.into());
                    self.close();
                } else {
                    self.error = String::from("Invalid Jid");
                }
            },
        };

        Task::none()
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        
        let content = container(column![
            text("Add Contact"),
            text_input("jid", &self.text_input_jid)
                .on_input(Message::TextInputJid)
                .on_submit(Message::Submit),
            text(self.error.clone()),
            button("Submit")
                .on_press(Message::Submit),
        ]
            .spacing(10)
        )
            .width(300)
            .padding(20)
            .style(|_theme| {
                container::Style {
                    background: Some(Background::Color(Color::from_rgb8(50, 50, 200))),
                    ..Default::default()
                }
            });
        
        let base = None::<Element<'a, Message>>;
        modal(base, content, Message::Close)
    }

    fn close(&mut self) {
        self.error.clear();
        self.text_input_jid.clear();
        self.show = false;
    }
}

use crate::ui::dashboard::Event;

impl From<Message> for Event {
    fn from(value: Message) -> Self {
        Event::AddContactModal(value)
    }
}
