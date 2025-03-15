use async_channel::Sender;
use crate::core::event::UiEvent;
use futures::StreamExt;
use iced::widget::{self, *, column};
use iced::*;
use tokio_xmpp::jid::Jid;
use xmpp_parsers::presence::Presence;
use super::modal;


#[derive(Debug, Clone)]
pub enum Message {
    Open,
    Close,
    Submit,
    TextInputJid(String),
    TextInputPassword(String),
    LoginFailed(String),
    LoginSuccesful(Jid),
}

pub struct AddAccount {
    pub show: bool, 
    sender: Sender<UiEvent>,
    pub text_input_jid: String,
    pub text_input_password: String,
    error: String,
    submit_pending: bool,
}

impl AddAccount {

    pub fn new(sender: Sender<UiEvent>) -> Self {
        Self {
            sender,
            show: false,
            text_input_jid: String::new(),
            text_input_password: String::new(),
            error: String::new(),
            submit_pending: false,
        }
    }

    pub fn update<F>(
        &mut self, 
        message: Message,
        mut new_account_jid: F 
    ) -> Task<Message> 
    where
        F: FnMut(Jid)
    {
        match message {
            Message::Open => { 
                self.show = true;
                return widget::focus_next();
            },
            Message::Close => { 
                self.text_input_jid.clear();
                self.text_input_password.clear();
                self.submit_pending = false;
                self.error.clear();
                self.show = false;
            },
            Message::TextInputJid(s) => {
                self.text_input_jid = s;

            }
            Message::TextInputPassword(s) => {
                self.text_input_password = s;
            }
            Message::Submit => {
                self.submit_pending = true;
                
                // TODO insteal of unwrap, show error
                let jid = Jid::new(&self.text_input_jid).unwrap(); 
                // TODO show error when not provided with a node
               
                let password = self.text_input_password.clone();
            
                return Task::perform(
                    new_xmpp_client(jid, password, self.sender.clone()),
                    |result| { 
                        match result {
                            Ok(jid) => Message::LoginSuccesful(jid),
                            //Ok(jid) => { 
                            //    new_accound_jid(jid); 
                            //    Message::Close 
                            //}
                            Err(err) => Message::LoginFailed(err),
                        }
                    }
                );
            }
            Message::LoginSuccesful(jid) => {
                new_account_jid(jid);
                return self.update(Message::Close, new_account_jid);
            }
            Message::LoginFailed(err) => {
                self.error = err;
                self.submit_pending = false;
            },
            //_ => { unreachable!() }
        };

        Task::none()
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        
        let content = container(column![
            text("Add Account"),
            text_input("jid", &self.text_input_jid)
                .on_input_maybe(if self.submit_pending { None } else { Some(Message::TextInputJid) } ),
            text_input("password", &self.text_input_password)
                .on_input_maybe(if self.submit_pending { None } else { Some(Message::TextInputPassword) }),
            text(self.error.clone()),
            button("Sumbit")
                .on_press_maybe(if self.submit_pending { None } else { Some(Message::Submit) }),
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
        
        content.into()
        //let base = None::<Element<'a, Message>>;
        //modal(Some(base), content, Message::Close)
    }
}

async fn new_xmpp_client(
    jid: Jid, 
    password: String,
    sender: Sender<UiEvent>,
) -> std::result::Result<Jid, String> {

    use xmpp_parsers::presence::*;
    use tokio_xmpp::Client;

    let mut client = Client::new(jid, password.clone());

    let event = client.next().await.unwrap();

    match event {
        tokio_xmpp::Event::Online { bound_jid, resumed: _ } => {
            let presence = {
                let mut presence = Presence::new(Type::None);
                presence.show = Some(Show::Chat);
                presence
                    .statuses
                    .insert(String::from("en"), String::from("Echoing messages."));
                presence
            };
            
            client.send_stanza(tokio_xmpp::Stanza::Presence(presence)).await.unwrap();
             
            sender.send(UiEvent::NewXmppClient { 
                jid: bound_jid.clone(),
                password,
                client: client.into(),
            }).await.unwrap();

            Ok(bound_jid)
        },
        tokio_xmpp::Event::Disconnected(err) => {
            Err(err.to_string())
        },
        _ => { 
            unreachable!("next time pray harder.") 
        },
    }
}


//use crate::Event;
//use super::Event;
use crate::ui::dashboard::Event;

impl From<Message> for Event {
    fn from(value: Message) -> Self {
        Event::AddAccountModal(value)
    }
}
