use iced::Alignment::Center;
//use crate::core::Status;
use iced::alignment::Vertical;
use iced::border::Radius;
use iced::widget::*;
use iced::{Background, Border, Color, Point, Task, window};
use iced::{Element, Fill, Length, Theme};

use crate::core::Status;
use crate::core::event::{BackendEvent, UiEvent};
use iced::alignment::Horizontal;
use screen::dashboard;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::marker::PhantomData;
use std::sync::Arc;
use xmpp_parsers::jid::Jid;
//use crate::core::account::{AccountId, Account};
use crate::core::account::AccountId;
use async_channel::{Receiver, Sender};
use iced::widget::column;

mod screen;

use crate::core::account::Account;
use crate::core::contact::{Contact, ContactId};
use screen::dashboard::modal::{
    add_account::{AddAccount as AddAccountModal, Message as AddAcountModalMessage},
    add_contact::{AddContact as AddContactModal, Message as AddContactModalMessage},
};

//use crate::ui::dashboard;

#[derive(Debug)]
enum Message {
    Dashboard(dashboard::Event),
}

// TODO deal with name
struct Akama {
    dashboard: dashboard::State,
    dashboard_id: window::Id,
}

enum Screen {
    Dashboard(screen::dashboard::State),
}

impl Akama {
    fn new(reciever: Receiver<BackendEvent>, sender: Sender<UiEvent>) -> (Self, Task<Message>) {
        let dashboard = screen::dashboard::State::new(reciever.clone(), sender);
        let (id, open) = window::open(window::Settings::default());

        (
            Self {
                dashboard,
                dashboard_id: id,
            },
            Task::batch([
                open.then(|f| Task::none()),
                Task::run(reciever, |event| {
                    Message::Dashboard(Event::BackendEvent(event))
                }),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Dashboard(message) => {
                return self
                    .dashboard
                    .update(message)
                    .map(|message| Message::Dashboard(message));
            }
        }

        Task::none()
    }

    pub fn view(&self, id: window::Id) -> Element<Message> {
        if id == self.dashboard_id {
            self.dashboard.view().map(Message::Dashboard)
        } else {
            text("hi").into()
        }
    }
}

use crate::ui::dashboard::*;

pub fn run(reciever: Receiver<BackendEvent>, sender: Sender<UiEvent>) {
    iced::daemon("title", Akama::update, Akama::view)
        .theme(|_state, _id| Theme::Dark)
        //.theme(State::theme)
        .run_with(move || Akama::new(reciever.clone(), sender))
        .unwrap();
}
