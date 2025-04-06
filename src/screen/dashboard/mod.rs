use container::background;
use iced::*;
use iced::widget::*;
use log::*;
use iced::Alignment::Center;
//use crate::core::Status;
use iced::border::Radius;
use iced::{window, Background, Border, Color,Task};
use iced::{Element, Fill, Length, Theme};
use iced::alignment::Horizontal;
use modal::add_account;
use std::collections::HashMap;
use tokio_xmpp::parsers::jid::Jid;
use async_channel::Sender;
use iced::widget::column;
use crate::common::{BackendEvent, UiEvent, AccountId, Account, Contact, ContactId, Status};
// use crate::screen::dashboard;
use crate::screen::dashboard::modal::{
    add_account::{AddAccount as AddAccountModal, Message as AddAcountModalMessage},
    add_contact::{AddContact as AddContactModal, Message as AddContactModalMessage},
};

pub mod modal;
// pub mod style;

pub struct State {
    sender: Sender<UiEvent>,
    //pub reciever: RefCell<Option<Receiver<Event>>>,
    theme: Theme,
    input_value: String,

    accounts: HashMap<AccountId, Account>,
    current_account: Option<ContactId>,
    contacts: HashMap<AccountId, HashMap<ContactId, Contact>>,
    current_contact: Option<ContactId>,

    new_message_text_input: String,
    selection: Option<String>,

    add_account_modal: AddAccountModal,
    add_contact_modal: AddContactModal,
}


#[derive(Debug, Clone)]
pub enum Message {
    Open(window::Id),
    ThemeChanged(Theme),

    BackendEvent(BackendEvent),

    AddAccountModal(modal::add_account::Message),
    AddContactModal(modal::add_contact::Message),

    ChangeCurrentAccount(AccountId),
    ChangeCurrentContact(ContactId),

    InputSubmit,
    InputChanged(String),
}

use std::convert::From;

impl State {
    pub fn new(sender: Sender<UiEvent>) -> Self {
        State {

            add_account_modal: AddAccountModal::new(sender.clone()),
            add_contact_modal: AddContactModal::new(sender.clone()),
            sender,

            new_message_text_input: String::new(),
            //show_add_account_modal: false,
            theme: Theme::TokyoNight,
            input_value: String::new(),
            accounts: HashMap::new(),
            current_account: None,
            current_contact: None,
            contacts: HashMap::new(),
            selection: None,
            //show_add_contact_modal: false,
            //new_contact_text_input: String::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        trace!("new ui event {:?}", message);

        match message {
            Message::Open(_id) => {
                println!("focuss gained");
            }
            Message::BackendEvent(backend_event) => {
                if let Some(task) = self.update_backend(backend_event) {
                    return task;
                }
            }
            // handling modals
            Message::AddAccountModal(message) => {
                let r = self
                    .add_account_modal
                    .update(message, |jid| {
                        self.accounts.insert(
                            jid.clone().into(),
                            Account::new(jid.clone(), Status::Online(None)),
                        );
                    });
                if let Some(task) = r {
                    return task.map(|message| message.into());
                }
            }
            Message::AddContactModal(message) => {
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
            Message::ChangeCurrentAccount(index) => {
                self.current_account = Some(index);
            }
            Message::ChangeCurrentContact(jid) => {
                self.current_contact = Some(jid);
            }
            Message::InputSubmit => {
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
            Message::InputChanged(txt) => {
                self.new_message_text_input = txt;
            }
            _other => {
                warn!("unhandled event");
            }
        };

        Task::none()
    }

    fn update_backend(&mut self, backend: BackendEvent) -> Option<Task<Message>> {
        match backend {
            BackendEvent::Account(id, account) => {
                self.accounts.insert(id, account);
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
            BackendEvent::Message {
                to,
                from,
                body,
                by_me,
                timestamp,
                id,
            } => {
                let contacts = if by_me {
                    self.contacts.get_mut(&from).unwrap()
                } else {
                    self.contacts.get_mut(&to).unwrap()
                };

                let contact = contacts.get_mut(if by_me { &to } else { &from }).unwrap();

                contact.new_text(body, by_me);
            }
            _ => warn!("unhandled backend event"),
        };
    
        None
        // Task::none()
        
    }

    //pub fn view(&self, id: window::Id) -> Element<Event> {
    pub fn view(&self) -> Element<Message> {

        // use iced_aw::{menu_bar, menu_items};

        let mut out = Row::new()
            .width(Length::Fill)
            .height(Length::Fill);

        out = out
            .push(self.account_panel())
            .push_maybe(self.contact_panel());
        
        if let Some(chat_panel) = self.chat_panel() {
            out = out.push(chat_panel);
        } else {
            out = out.push_maybe(self.account_status_panel());
        }

        if self.add_account_modal.show {
            let view = self.add_account_modal.view().map(|m| m.into());
            let modal = modal(out.into(), view, AddAcountModalMessage::Close.into());
            return modal;
        } else if self.add_contact_modal.show {
            let view = self.add_contact_modal.view().map(|message| message.into());
            let modal = modal(out.into(), view, AddContactModalMessage::Close.into());
            return modal;
        }

        container(out)
            .style(|style| container::Style {
                background: Some(Background::Color(color!(0x12142b))),
                ..Default::default()
            })
            .into()
    }



    fn account_panel(&self) -> Element<Message> {

        let mut panel = column!()
            .width(70)
            .height(Fill)
            .padding(10)
            .spacing(10);

        for (id, _) in &self.accounts {
            
            let is_selected = self.current_account.as_ref().is_some_and(|a| a == id);

            let first_letter = id.as_str()[0..1].to_string();

            let txt = text(first_letter)
                .align_x(Center)
                .align_y(Center);

            let btn = button(txt)
                .on_press(Message::ChangeCurrentAccount(id.clone()))
                .width(50)
                .height(50)
                .style(move |_theme, _status| {
                    let mut bg = if is_selected { color!(0, 155, 0) } else { color!(0, 255, 0) };
                    button::Style {
                        background: Some(bg.into()),
                        ..Default::default()
                    }
                });
            panel = panel.push(btn);
        }

        let add_btn = button(text("+").center())
            .on_press(Message::AddAccountModal(AddAcountModalMessage::Open))
            .width(50)
            .height(50)
            .style(|_theme, _status| {
                button::Style {
                    background: Some(color!(0, 255, 0).into()),
                    ..Default::default()
                }
            });

        panel = panel.push(add_btn);

        let panel = container(panel)
            .style(|_theme| {
                container::Style {
                    background: Some(color!(0x181b39).into()), 
                    ..Default::default()
                }
            });


        panel.into()
    }

    fn contact_panel(&self) -> Option<Element<Message>> {
    
        let mut c_list = column!()
            .width(340)
            .height(Fill)
            // .spacing(-1.0)
            // .spacing(10)
            // .padding(10);
            ;

        let current_account = self.current_account.clone()?;
         
        // let cur_contact = self.contacts.get()
        if let Some(contacts) = self.contacts.get(&current_account) {
            for (jid, _contact) in contacts {

                let t = text(jid.to_string())
                    .size(17)
                    .color(Color::WHITE);

                let b = button(t)
                    .on_press(Message::ChangeCurrentContact(jid.clone()))
                    .width(Fill)
                    .padding(24)
                    // .padding(Padding { top: 20.0, bottom: 20.0, left: 10.0, right: 10.0})
                    .style(|_theme, state| { 
                        
                        let bg = if matches!(state, button::Status::Hovered) {
                            color!(0x15183d)
                            // color!(0x999999, 0.1)
                        } else {
                            Color::TRANSPARENT
                        };

                        button::Style { 
                            background: Some(bg.into()),
                            // border: Border { color: color!(0x666666), width: 1.0, radius: 0.0.into() },
                            ..Default::default()
                        }}
                    )
                    ;

                c_list = c_list.push(b);

                let br = container("")
                    .width(Fill)
                    .height(1)
                    .style(|_theme| {
                        container::Style { 
                            background: Some(color!(0x666666).into()),
                            ..Default::default()
                        }
                    });

                c_list = c_list.push(br);
            }
        }

        let add_c_btn = button(text("+").center().width(Fill).size(17))
            .on_press(AddContactModalMessage::Open.into())
            .width(Fill)
            .padding(24)
            .style(|_theme, _status| {
                button::Style {
                    text_color: Color::WHITE,
                    background: Some(Color::TRANSPARENT.into()),
                    // border: Border { color: color!(0x666666), width: 1.0, radius: 0.0.into() },
                    ..Default::default()
                }
            });
        c_list = c_list.push(add_c_btn);

        let br = container("")
            .width(Fill)
            .height(1)
            .style(|_theme| {
                container::Style { 
                    background: Some(color!(0x666666).into()),
                    ..Default::default()
                }
            });
        c_list = c_list.push(br);

        let c_panel = container(c_list)
            .height(Fill)
            // .style(|_theme| container::Style {
            //     border: Border {
            //         color: Color::from_rgb8(255, 0, 0),
            //         width: 1.0,
            //         radius: Radius::new(10),
            //     },
            //     ..Default::default()
            // })
            ;

        // using container as border because, it dont allow for one sided border (i think) 
        let border = container("")
            .width(1)
            .height(Fill)
            .style(|_theme| {
                container::Style {
                    background: Some(color!(0x666666).into()),
                    ..Default::default()
                }
            });
        let border2 = container("")
            .width(1)
            .height(Fill)
            .style(|_theme| {
                container::Style {
                    background: Some(color!(0x666666).into()),
                    ..Default::default()
                }
            });
        

        let c_panel = row![border, c_panel, border2];

        Some(c_panel.into())
    }

    fn chat_panel(&self) -> Option<Element<Message>> {
        
        let current_contact = self.current_contact.as_ref()?;       

        let mut msg_col = Column::new()
            .padding(20)
            .spacing(4);

        let account_id = self.current_account.as_ref().unwrap();

        let map = self.contacts.get(account_id)?;
        let contact = map.get(&current_contact)?;

        for msg in &contact.chat_history {

            let content = match &msg.content {
                crate::common::message::Content::Text(t) => t,
            };

            let x_align = if msg.by_me { Horizontal::Left } else { Horizontal::Right };

            let txt = text(content)
                // .align_x(x_align)
                .width(Shrink)
                ;

            let txt = container(txt)
                .max_width(400)
                .padding(Padding { top: 10.0, right: 16.0, bottom: 10.0, left: 16.0 })
                .style(|_theme| {
                    container::Style {
                        background: Some(color!(0x181b39).into()),
                        border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 10.0.into() },
                        ..Default::default()
                    }
                });
            
            
            let txt = container(txt)
                .width(Fill)
                .align_x(x_align)
                ;

            msg_col = msg_col.push(txt);
            //
            // let con = container(text(format!("{}", msg.stamp)))
            //     .align_x(x_align)
            //     .width(Fill);

            // let element = Element::new(
            //     container(column![
            //         txt,
            //         con,
            //     ])
            //     .padding(10)
            //     .width(Length::Shrink)
            //     .style(|_theme| container::Style {
            //         border: Border {
            //             color: Color::from_rgb8(20, 20, 250),
            //             radius: Radius::new(1),
            //             width: 1.0,
            //         },
            //         ..Default::default()
            //     }),
            // );
            // msg_col = msg_col.push(element);
        }

        let msg_pannel = container(scrollable(msg_col).anchor_bottom())
            .width(Fill)
            .height(Fill);

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
                    Some(Message::InputChanged)
                })
                .on_submit(Message::InputSubmit),
        );

        // out = out.push(msg_pannel);
        Some(msg_pannel.into())
        
        // } else if let Some(current_account) = &self.current_account {
        //     let account = self.accounts.get(current_account).unwrap();
        //
        //     let content = container(
        //         column![
        //             text(current_account.to_string()),
        //             text(format!("Status: {:?}", account.status()))
        //         ]
        //         .spacing(20),
        //     )
        //     .width(Fill)
        //     .height(Fill)
        //     .align_x(Center)
        //     .align_y(Center);
        //
        //     return Some(content.into()
        //
        //     // out = out.push(content);
        // };

        // None
    }
    
    fn account_status_panel(&self) -> Option<Element<Message>> {
        let id = self.current_account.as_ref()?; 
        let a = self.accounts.get(id).unwrap();
    
        let status = text(format!("Status: {}", a.status()));
        let id = text(id.to_string());
        
        let col = column![id, status]
            .width(Shrink)
            .height(Shrink)
            .spacing(20)
            .padding(20);

        let r = container(col)
            .width(Fill)
            .height(Fill)
            .align_x(Center)
            .align_y(Center);

        Some(r.into())
    }

    /// gets the contacts of current account
    fn current_contacts<'a>(&'a self) -> Option<&'a HashMap<ContactId, Contact>> {
        let current_account = self.current_account.as_ref()?;
        let contacts = self.contacts.get(&current_account)?;
        Some(contacts)
    }

    pub fn theme(&self) -> Theme {
        //self.theme.clone()
        Theme::Dark
    }
}

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

// fn border_con<Message>() -> Element<Message> {
//     container("")
//     .style(|_theme| container::Style{)
// }
