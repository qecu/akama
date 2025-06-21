use iced::*;
use iced::widget::*;
use async_channel::{Receiver, Sender};
use dashboard::Message as DashboardMessage;
use crate::common::{BackendEvent, UiEvent};
pub mod dashboard;


#[derive(Debug)]
enum Message {
    Dashboard(dashboard::Message),
}

// TODO deal with name
struct Akama {
    dashboard: dashboard::State,
    dashboard_id: window::Id,
}

enum Screen {
    Dashboard(dashboard::State),
}

impl Akama {
    fn new(reciever: Receiver<BackendEvent>, sender: Sender<UiEvent>) -> (Self, Task<Message>) {
        let dashboard = dashboard::State::new(sender);
        let (id, open) = window::open(window::Settings::default());

        (
            Self {
                dashboard,
                dashboard_id: id,
            },
            Task::batch([
                open.then(|_f| Task::none()),
                Task::run(reciever, |event| {
                    Message::Dashboard(DashboardMessage::BackendEvent(event))
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
        // Task::none()
    }

    pub fn view(&self, id: window::Id) -> Element<Message> {
        // iced::widget::radio(jjh, value, selected, on_click)
        if id == self.dashboard_id {
            self.dashboard.view().map(Message::Dashboard)
        } else {
            text("hi").into()
        }
    }
}

pub fn run(reciever: Receiver<BackendEvent>, sender: Sender<UiEvent>) {
    iced::daemon("title", Akama::update, Akama::view)
        .theme(|_state, _id| Theme::Dark)
        //.theme(State::theme)
        .run_with(move || Akama::new(reciever.clone(), sender))
        .unwrap();
}
