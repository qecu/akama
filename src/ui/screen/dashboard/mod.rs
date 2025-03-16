pub mod modal;

use iced::Alignment::Center;
//use crate::core::Status;
use iced::alignment::Vertical;
use iced::border::Radius;
use iced::widget::*;
use iced::{Background, Border, Color, Task, window};
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

use crate::ui::screen;

use crate::core::account::Account;
use crate::core::contact::{Contact, ContactId};
use screen::dashboard::modal::{
    add_account::{AddAccount as AddAccountModal, Message as AddAcountModalMessage},
    add_contact::{AddContact as AddContactModal, Message as AddContactModalMessage},
};

//use crate::ui::{State, Event};

pub struct State {
    sender: Sender<UiEvent>,
    //pub reciever: RefCell<Option<Receiver<Event>>>,
    pub theme: Theme,
    pub input_value: String,
    //pub chats: Vec<Message>,
    accounts: HashMap<AccountId, Account>,
    current_account: Option<ContactId>,
    current_contact: Option<ContactId>,
    contacts: HashMap<AccountId, HashMap<ContactId, Contact>>,

    //show_add_contact_modal: bool,
    //show_add_account_modal: bool,

    // new_contact_text_input: String,
    new_message_text_input: String,

    // new_account_text_input: String,

    //text_input_account_password: String,
    //text_input_account_jid: String,
    add_account_modal: AddAccountModal,
    add_contact_modal: AddContactModal,
    //contactts: HashMap<(AccountId, ContactId), Contact>,
}

#[derive(Debug, Clone)]
pub enum Event {
    Open(window::Id),
    Hi,
    ThemeChanged(Theme),
    InputChanged(String),
    InputSubmit,

    //NewXmppMessage(Message),
    ChangeCurrentAccount(ContactId),
    ChangeCurrentContact(ContactId),
    NewAccount(Jid),
    AccountButton(u32),
    AccountOnline(Jid),
    AccountDisconnected(Jid),
    NewContact { account_id: Jid, contact_id: Jid },

    BackendEvent(BackendEvent),

    LogIn { jid: Jid, password: String },

    //TODO remove these
    ShowAddContactModal,
    HideAddContectModal,
    //AddContactInput(String),
    AddContactSubmit,
    AddContactTextInput(String),

    NewMessageSubmit,
    NewMessageTextInput(String),

    AddAccountTextInput(String),
    AddAccountSubmit,

    // TODO remove these
    ShowAddAccountModal,
    HideAddAccountModal,
    TextInputAccountJid(String),
    TextInputAccountPassword(String),
    SubmitAccount,

    AddAccountModal(dashboard::modal::add_account::Message),
    AddContactModal(dashboard::modal::add_contact::Message),
}

use std::convert::From;

impl State {
    pub fn new(reciever: Receiver<BackendEvent>, sender: Sender<UiEvent>) -> Self {
        State {
            add_account_modal: AddAccountModal::new(sender.clone()),
            add_contact_modal: AddContactModal::new(sender.clone()),

            //text_input_account_jid: String::new(),
            //text_input_account_password: String::new(),

            //new_account_text_input: String::new(),
            sender,
            new_message_text_input: String::new(),
            //show_add_account_modal: false,
            theme: Theme::TokyoNight,
            input_value: String::new(),
            accounts: HashMap::new(),
            current_account: None,
            current_contact: None,
            contacts: HashMap::new(),
            //show_add_contact_modal: false,
            //new_contact_text_input: String::new(),
        }
    }

    pub fn update(&mut self, message: Event) -> Task<Event> {
        log::trace!("new ui event {:?}", message);

        //window::open(window::Settings::default());

        match message {
            Event::Open(id) => {
                println!("focuss gained");
                //window::open
                //return window::change_mode(id, window::Mode::Windowed);
                //return window::gain_focus(id);
            }
            // handling modals
            Event::AddAccountModal(message) => {
                return self
                    .add_account_modal
                    .update(message, |jid| {
                        self.accounts.insert(
                            jid.clone().into(),
                            Account::new(jid.clone(), Status::Online),
                        );
                    })
                    .map(|message| message.into());
            }
            Event::AddContactModal(message) => {
                return self
                    .add_contact_modal
                    .update(message, |contact_jid| {
                        let contacts = self
                            .contacts
                            .entry(self.current_account.clone().unwrap())
                            .or_insert(HashMap::new());

                        contacts.insert(
                            contact_jid.clone(),
                            Contact::new(contact_jid.clone(), Status::Offline, Vec::new()),
                        );

                        self.sender
                            .send_blocking(UiEvent::NewContact(
                                self.current_account.clone().unwrap(),
                                contact_jid.clone(),
                            ))
                            .unwrap();
                    })
                    .map(|message| message.into());
            }

            Event::BackendEvent(backend_event) => {
                match backend_event {
                    BackendEvent::Account(id, account) => {
                        self.accounts.insert(id, account);
                    }
                    BackendEvent::AccountOnline(id, new_jid) => {
                        let jid = self.accounts.get_mut(&id).unwrap();
                        jid.set_status(Status::Online);
                        jid.set_jid(new_jid);
                    }
                    BackendEvent::AccountStatusUpdate(id, new_status) => {
                        self.accounts.get_mut(&id).unwrap().set_status(new_status);
                    }
                    BackendEvent::Contacts(contacts) => {
                        for (account, contact, chats) in contacts {
                            let contacts = self.contacts.entry(account).or_default();
                            contacts.insert(contact, chats);
                        }
                    }
                    BackendEvent::Contact {
                        account,
                        contact,
                        chats,
                    } => {
                        todo!()
                        //self.contacts
                        //    .get_mut(&account)
                        //    .unwrap()
                        //    .insert(contact, chats)
                        //;
                    }
                    BackendEvent::Message {
                        to,
                        from,
                        body,
                        by_me,
                        timestamp,
                    } => {
                        let contacts = if by_me {
                            self.contacts.get_mut(&from).unwrap()
                        } else {
                            self.contacts.get_mut(&to).unwrap()
                        };

                        let contact = contacts.get_mut(if by_me { &to } else { &from }).unwrap();

                        contact.new_text(body, by_me);
                    }
                };
            }
            Event::ChangeCurrentAccount(index) => {
                self.current_account = Some(index);
            }
            Event::ChangeCurrentContact(jid) => {
                self.current_contact = Some(jid);
            }
            //Event::ThemeChanged(theme) => {
            //    self.theme = theme;
            //}
            Event::AddContactSubmit => {
                //let contact_jid = Jid::new(&self.new_contact_text_input).unwrap();
                //let current_acc = self.current_account.as_ref().expect("this shouldn't be none, something went wrong");
                //
                //let contacts_map = match self.contacts.get_mut(&current_acc) {
                //    Some(v) => v,
                //    None => {
                //        self.contacts.insert(current_acc.clone(), HashMap::new());
                //        self.contacts.get_mut(&current_acc).unwrap()
                //    }
                //};
                //contacts_map.insert(contact_jid.clone().to_bare(), Contact::new(contact_jid.to_bare(), Status::Offline, Vec::new()));
                //
                //// TODO maybe spawn a task then block
                ////self.sender.send_blocking(UiEvent::NewContact(
                ////    self.current_account.clone().unwrap(),
                ////    Jid::new(self.new_contact_text_input.as_str()).unwrap().to_bare()
                ////)).unwrap();
                //
                //self.new_contact_text_input.clear();
                //return self.update(Event::HideAddContectModal);
            }
            Event::NewMessageSubmit => {
                //todo!()
                let from = self.current_account.clone().unwrap();
                let to = self.current_contact.clone().unwrap();
                self.sender
                    .send_blocking(UiEvent::NewText {
                        from,
                        to,
                        content: self.new_message_text_input.clone(),
                    })
                    .unwrap();
                self.new_message_text_input.clear();
                //self.sender.send_blocking(UiEvent::NewTextMessage {
                //    from: self.current_account.unwrap().clone(),
                //    to: self.current_contact.clone().unwrap(),
                //    content: self.new_message_text_input.clone()
                //}).unwrap();
                //self.new_message_text_input.clear();
            }
            Event::NewMessageTextInput(txt) => {
                self.new_message_text_input = txt;
            }

            other => {
                println!("other events: {:?}", other)
            }
        };

        Task::none()
    }

    //pub fn view(&self, id: window::Id) -> Element<Event> {
    pub fn view(&self) -> Element<Event> {
        use iced_aw::menu::Item;
        use iced_aw::menu::Menu;
        use iced_aw::{menu_bar, menu_items};

        //let menu_tpl_1 = |items| Menu::new(items).max_width(180.0).offset(15.0).spacing(5.0);

        //let menut = Menu::new()

        let menut = Menu::new(menu_items!((button("sssss").width(200))))
            .max_width(150.0)
            .offset(0.0)
            .spacing(5.0);

        let menu = menu_bar!((button("sdf"), menut));

        println!("something view");
        let mut out = row![].width(Length::Fill).height(Length::Fill);

        let account_pannel = container(
            column(self.accounts.iter().map(|(jid, acc)| {
                let first_letter = jid.as_str()[0..1].to_string();
                let current_acc = self.current_account.clone();
                Element::new(
                    button(
                        text(first_letter)
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center),
                    )
                    .on_press(Event::ChangeCurrentAccount(jid.clone()))
                    .width(50)
                    .height(50)
                    .style(move |theme, status| {
                        if let Some(cur_acc) = &current_acc {
                            if cur_acc == jid {
                                return button::Style {
                                    background: Some(Background::Color(Color::from_rgb8(
                                        100, 100, 0,
                                    ))),
                                    ..Default::default()
                                };
                            }
                        };
                        button::primary(theme, status)
                    }),
                )
            }))
            .push(
                button(text("+").align_x(Horizontal::Center))
                    .on_press(AddAcountModalMessage::Open.into()),
            )
            .spacing(20)
            .width(70)
            .height(Fill)
            .padding(10),
        )
        .style(|_theme| container::Style {
            border: Border {
                color: Color::from_rgb8(0, 255, 0),
                width: 1.0,
                radius: Radius::new(1),
            },
            ..Default::default()
        });
        out = out.push(account_pannel);

        if let Some(current_acc) = &self.current_account {
            let mut contact_column = column!().width(300).height(Fill).padding(10);
            if let Some(contacts) = self.contacts.get(current_acc) {
                //println!("yes contacts {:?}", contacts);
                for (jid, _contact) in contacts {
                    //println!("contacts listt {}", jid);
                    let cur_contact = self.current_contact.clone();
                    let button = Element::new(
                        button(text(jid.to_string()))
                            .on_press(Event::ChangeCurrentContact(jid.clone()))
                            .width(Fill)
                            .padding(10)
                            .style(move |_theme, _state| {
                                let mut background =
                                    Some(Background::Color(Color::from_rgba8(200, 200, 200, 0.2)));

                                if let Some(cur_con) = &cur_contact {
                                    if cur_con == jid {
                                        background = Some(Background::Color(Color::from_rgba8(
                                            200, 200, 200, 0.8,
                                        )));
                                    }
                                };
                                // TODO a color for hovering
                                button::Style {
                                    //background: Some(Background::Color(Color::from_rgba8(200, 200, 200, 0.2))),
                                    background,
                                    border: Border {
                                        color: Color::TRANSPARENT,
                                        width: 0.0,
                                        radius: Radius::new(10.0),
                                    },
                                    ..Default::default()
                                }
                            }),
                    );
                    contact_column = contact_column.push(button)
                }
            }
            contact_column =
                contact_column.push(button("+").on_press(AddContactModalMessage::Open.into()));

            let contact_pannel = container(contact_column).style(|_theme| container::Style {
                border: Border {
                    color: Color::from_rgb8(255, 0, 0),
                    width: 1.0,
                    radius: Radius::new(10),
                },
                ..Default::default()
            });
            out = out.push(contact_pannel);
        }

        if let Some(current_contact) = &self.current_contact {
            let mut msg_col = Column::new().padding(20).spacing(4);

            let account_id = self.current_account.as_ref().unwrap();
            let map = self.contacts.get(account_id).unwrap();
            let contact = map.get(current_contact).unwrap();
            for msg in &contact.chat_history {
                let content = match &msg.content {
                    crate::core::message::Content::Text(t) => t,
                };

                //let left = if msg == *self.current_account.as_ref().unwrap() { true } else { false };

                let element = Element::new(
                    container(column![
                        text(content)
                            .align_x(if msg.by_me {
                                Horizontal::Left
                            } else {
                                Horizontal::Right
                            })
                            .width(Fill),
                        container(text(format!("{}", msg.stamp)))
                            //.width(Length::Fill)
                            .style(|theme| {
                                container::Style {
                                    border: Border {
                                        color: Color::from_rgb8(255, 0, 0),
                                        width: 1.0,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            }),
                    ])
                    .padding(10)
                    .width(Length::Shrink)
                    .style(|_theme| container::Style {
                        border: Border {
                            color: Color::from_rgb8(20, 20, 250),
                            radius: Radius::new(1),
                            width: 1.0,
                        },
                        ..Default::default()
                    }),
                );
                msg_col = msg_col.push(element);
            }

            let msg_pannel = container(msg_col).width(Fill).height(Fill);

            let mut disable_message_box = false;

            //let ss = Column::with_children([text("sdf")]);
            //self.accounts.get(self.current_account.unwrap())
            let mut msg_pannel = column![msg_pannel,];
            if let Some(acc) = &self.current_account {
                if let Some(acc) = self.accounts.get(acc) {
                    match acc.status() {
                        Status::Offline => {
                            msg_pannel =
                                msg_pannel.push(text("You need to be online to send messages..."));
                            disable_message_box = true;
                        }
                        Status::Connecting => {
                            msg_pannel = msg_pannel
                                .push(text("still connecting. you can message when you're online"));
                            disable_message_box = true;
                        }
                        _ => {}
                    }
                }
            }
            msg_pannel = msg_pannel.push(
                text_input("message here...", &self.new_message_text_input)
                    .on_input_maybe(if disable_message_box {
                        None
                    } else {
                        Some(Event::NewMessageTextInput)
                    })
                    .on_submit(Event::NewMessageSubmit),
            );

            out = out.push(msg_pannel);
        } else if let Some(current_account) = &self.current_account {
            let account = self.accounts.get(current_account).unwrap();

            let content = container(
                column![
                    text(current_account.to_string()),
                    text(format!("Status: {:?}", account.status()))
                ]
                .spacing(20),
            )
            .width(Fill)
            .height(Fill)
            .align_x(Center)
            .align_y(Center);

            out = out.push(content);
        };

        if self.add_account_modal.show {
            let view = self.add_account_modal.view().map(|m| m.into());
            let modal = modal(out.into(), view, AddAcountModalMessage::Close.into());
            return modal;
        } else if self.add_contact_modal.show {
            let view = self.add_contact_modal.view().map(|message| message.into());
            let modal = modal(out.into(), view, AddContactModalMessage::Close.into());
            return modal;
        }

        column!(menu, out).into()
        //menu.into()
        //out.into()
    }

    pub fn theme(&self) -> Theme {
        //self.theme.clone()
        Theme::Dark
    }
}

//pub fn run(
//    reciever: Receiver<BackendEvent>,
//    sender: Sender<UiEvent>
//) {
//
//    iced::application("styling - iced", State::update, State::view)
//        .theme(State::theme)
//        .run_with(|| {
//            (
//                State {
//                    add_account_modal: AddAccountModal::new(sender.clone()),
//                    add_contact_modal: AddContactModal::new(sender.clone()),
//
//                    //text_input_account_jid: String::new(),
//                    //text_input_account_password: String::new(),
//
//                    //new_account_text_input: String::new(),
//                    sender,
//                    new_message_text_input: String::new(),
//                    //show_add_account_modal: false,
//                    theme: Theme::TokyoNight,
//                    input_value: String::new(),
//                    accounts: HashMap::new(),
//                    current_account: None,
//                    current_contact: None,
//                    contacts: HashMap::new(),
//                    //show_add_contact_modal: false,
//                    //new_contact_text_input: String::new(),
//                },
//                Task::batch([
//                    Task::run(reciever, |event| Event::BackendEvent(event)),
//                ]),
//            )
//        }).unwrap();
//
//}

fn modal<'a, Message>(
    base: Option<impl Into<Element<'a, Message>>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let mut stack = stack![];
    stack = stack.push_maybe(base);
    stack =
        //base.into(),
        stack.push(
            opaque(
                mouse_area(center(opaque(content)).style(|_theme| {
                    container::Style {
                        background: Some(
                            Color {
                                a: 0.8,
                                ..Color::BLACK
                            }
                            .into(),
                        ),
                        ..container::Style::default()
                    }
                }))
                .on_press(on_blur)
            )
        );

    stack.into()
}
