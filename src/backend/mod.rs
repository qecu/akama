use crate::core::account::Account;
use crate::core::message::Message;
use crate::core::{AccountId, Status};
use crate::core::{
    contact::Contact,
    event::{BackendEvent, UiEvent},
};
use async_channel::{Receiver, Sender};
use chrono::Utc;
use futures::sink::SinkExt;
use futures::stream::Map;
use futures::stream::SelectAll;
use futures::stream::StreamExt;
use futures::stream::{SplitSink, SplitStream};
use log::info;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio_xmpp::connect::starttls::StartTlsClient as XmppTlsClient;
use tokio_xmpp::{Client, Event as TokioXmppEvent, Stanza, jid::Jid};
use uuid::Uuid;
use xmpp_parsers::jid::BareJid;
use xmpp_parsers::legacy_omemo::Payload;
use xmpp_parsers::message::{Body, Message as StanzaMessage, MessageType};
use xmpp_parsers::minidom::Element;
use xmpp_parsers::presence::{Presence, Show as PresenceShow, Type as PresenceType};

mod db;

pub async fn run(receiver: Receiver<UiEvent>, sender: Sender<BackendEvent>) {
    db::setup().await.unwrap();
    let accounts = db::get_accounts().await;

    let mut streams = SelectAll::new();
    let mut sinks = HashMap::new();

    for (jid, password) in accounts {
        log::debug!("accounts: {:#?}", jid);

        let xmpp_client = Client::new((*jid).clone(), password);

        let (client_sink, client_stream) = xmpp_client.split();
        streams.push(stream_to_jid_stream(client_stream, jid.clone()));
        sinks.insert(jid.clone(), client_sink);

        sender
            .send(BackendEvent::Account(
                jid.clone(),
                Account::new((**jid).clone(), Status::Connecting),
            ))
            .await
            .unwrap();

        //sender.send(BackendEvent::Contacts(jid.clone(), contacts.clone())).await.unwrap();

        let contacts = db::get_contacts(&jid).await;
        let mut contactsss = Vec::new();

        for contact in contacts {
            let chats = db::get_messages(&jid, &contact)
                .await
                .into_iter()
                .map(|(by_me, body, timestmap)| Message::new_text(body, by_me, timestmap))
                .collect();

            //subscribe(Jid::from((*contact).clone()));

            contactsss.push((
                jid.clone(),
                contact.clone(),
                Contact::new(contact.clone(), Status::Offline, chats),
            ));
        }

        if contactsss.is_empty() == false {
            sender
                .send(BackendEvent::Contacts(contactsss))
                .await
                .unwrap();
        }
    }

    loop {
        tokio::select! {
            event = streams.next() => {
                if event.is_none() {
                    continue
                }
                let (jid, event) = event.unwrap();
                match event {
                    TokioXmppEvent::Online { bound_jid, resumed: _ } => {
                        online(jid, bound_jid, sender.clone(), &mut sinks).await;
                    },
                    TokioXmppEvent::Disconnected(err) => {
                        println!("disconnecting");
                        sender.send(BackendEvent::AccountStatusUpdate(jid.clone(), Status::Offline)).await.unwrap();
                        sinks.get_mut(&jid).unwrap().close().await.unwrap();
                        sinks.remove(&jid);
                        log::info!("{} disconnected: {}", jid, err);
                    },
                    TokioXmppEvent::Stanza(stanza) => {
                        handle_stanza(stanza, sender.clone(), &mut sinks).await;
                    },
                }
            },
            ui_event = receiver.recv() => {
                let ui_event = ui_event.unwrap();
                handle_ui_event(&sender, ui_event, &mut sinks).await;
            }
        }
    }
}

async fn online(
    jid: AccountId,
    bound_jid: Jid,
    sender: Sender<BackendEvent>,
    sinks: &mut HashMap<AccountId, SplitSink<XmppTlsClient, Stanza>>,
) {
    log::info!("account online: {}", bound_jid);
    let presence = make_presence();
    sinks
        .get_mut(&jid)
        .unwrap()
        .send(presence.into())
        .await
        .unwrap();
    sender
        .send(BackendEvent::AccountOnline(jid.clone(), bound_jid))
        .await
        .unwrap();

    //sender.send(BackendEvent::AccountStatusUpdate(bound_jid, Status::Online)).await.unwrap();
    //sender.send()

    //let contacts = db::get_contacts(&jid).await;
    //for contact in contacts {
    //    let presence = subscribe(Jid::from((*contact).clone()));
    //    let sink = sinks.get_mut(&jid).unwrap();
    //    sink.send(Stanza::Presence(presence)).await.unwrap();
    //
    //}
}

async fn handle_ui_event(
    sender: &Sender<BackendEvent>,
    event: UiEvent,
    client_sinks: &mut HashMap<AccountId, SplitSink<XmppTlsClient, Stanza>>,
) {
    match event {
        UiEvent::NewText { from, to, content } => {
            let from_ = (*from).clone();
            let to_ = (*to).clone();

            let msg = new_text_stanza(from_, to_, content.clone());

            let sink = client_sinks.get_mut(&from).unwrap();
            sink.send(Stanza::Message(msg)).await.unwrap();

            let stamp = Utc::now();

            db::add_text_message(&from, to.as_ref(), content.as_str(), &stamp).await;

            sender
                .send(BackendEvent::Message {
                    from,
                    to,
                    body: content,
                    by_me: true,
                    timestamp: stamp,
                })
                .await
                .unwrap();
        }
        UiEvent::DisconnectAccount(_jid) => {
            //let sink = client_sinks.get(&jid).unwrap();
            // sink.close();

            //let sender = self.channels.get(&jid).unwrap();
            //sender.send(Command::Disconnect).await.unwrap();
        }
        //UiEvent::NewTextMessage { from, to, content } => {
        //    //let sender = self.channels.get(&from).unwrap();
        //    //sender.send(Comm)
        //}
        _ => {}
    };
}

fn new_text_stanza<J: Into<Jid>>(from: J, to: J, text: String) -> StanzaMessage {
    let mut bodies = BTreeMap::new();
    bodies.insert(String::new(), Body(text));

    // TODO handle paylaods
    //let payloads =

    let stanza = StanzaMessage {
        from: Some(from.into()),
        to: Some(to.into()),
        id: None,
        type_: MessageType::Chat,
        bodies,
        subjects: BTreeMap::new(),
        thread: None,
        payloads: vec![],
    };

    stanza
}

// Construct a <presence/>
fn make_presence() -> Presence {
    let mut presence = Presence::new(PresenceType::None);
    presence.show = Some(PresenceShow::Chat);
    presence
        .statuses
        .insert(String::from("en"), String::from("Echoing messages."));
    presence
}

fn subscribe(to: Jid) -> Presence {
    let mut presence = Presence::new(PresenceType::Subscribe);
    //presence.show = Some(PresenceShow::Chat);
    presence.to = Some(to);

    presence
}

fn stream_to_jid_stream(
    stream: SplitStream<XmppTlsClient>,
    jid: AccountId,
) -> Map<SplitStream<XmppTlsClient>, impl FnMut(TokioXmppEvent) -> (AccountId, TokioXmppEvent)> {
    stream.map(move |item| (jid.clone(), item))
}

async fn handle_stanza(
    stanza: Stanza,
    sender: Sender<BackendEvent>,
    client_sinks: &mut HashMap<AccountId, SplitSink<XmppTlsClient, Stanza>>, //db: Db,
) {
    match stanza {
        Stanza::Message(message) => {
            println!("message {:#?}", message);
            if let Some(body) = message.bodies.get("") {
                let from = message.from.clone().unwrap();
                let to = message.to.clone().unwrap();
                let body = body.0.to_owned();

                //let delay = message.payloads
                //    .iter()
                //    .find(|elm| elm.name() == "delay" )
                //    .unwrap_or_default();
                //
                //use chrono::prelude::*;
                ////use std::time::SystemTime;
                ////use human_time::ToHumanTimeString;
                //let stamp: chrono::DateTime<Utc> = stamp.parse().unwrap();
                //
                //db::add_new_message(&from, &to, &body, &stamp).await;

                //let now =
                //let diff =  - stamp.timestamp()
                //let stamp = chrono::Duration::from(stamp.signed_duration_since(std::time::UNIX_EPOCH));
                //let stamp = stamp.timestamp();
                //chrono::Duration::from(stamp);

                //let stmap SystemTime::from(stamp)
                //stamp.to_u
                db::add_text_message(&from.to_bare(), &to.to_bare(), body.as_str(), &Utc::now())
                    .await;

                //let now = time::now();
                //let relative = time::timestamp_to_relative_time(now);

                sender
                    .send(BackendEvent::Message {
                        to: to.into(),
                        from: from.into(),
                        body: body.clone(),
                        by_me: false,
                        timestamp: chrono::Utc::now(),
                    })
                    .await
                    .unwrap();

                let id = message.id.clone().unwrap();
                let re = recieved_elm(&id);
                let dis = displayed_elm(&id);

                //println!("my {:#?}\nmy {:#?}", re, dis);

                let mut bodies = BTreeMap::new();
                bodies.insert("".to_string(), Body(String::from("test11")));

                let msg = StanzaMessage {
                    from: message.to,
                    to: message.from,
                    id: Some(Uuid::new_v4().to_string()),
                    type_: MessageType::Chat,
                    bodies,
                    subjects: BTreeMap::new(),
                    thread: None,
                    //payloads: vec![re, dis],
                    payloads: Vec::new(),
                };
                //println!()

                //sinks.get_mut(&jid.to_bare()).unwrap().send(msg.into()).await.unwrap();

                //message.payloads.get()

                //sender.send(BackendEvent::NewMessage {
                //    from: from.clone(),
                //    to: to.clone(),
                //    //message: Message::new_text {content: Content::Text(String::from("sdf")) },
                //    message: Message::new_text(body.clone(), false),
                //    by_me: false
                //    //content:
                //}).await.unwrap();

                //db.new_text_message(from, to, body, stamp).unwrap();
            } else {
                //println!("{:#?}", message);
            }
        }
        Stanza::Iq(iq) => {
            println!("iq: {:#?}", iq);
        }
        Stanza::Presence(presence) => {
            println!("presece: {:#?}", presence);
        }
    }
    //match stanza {
    //    Stanza::Message(message) => {
    //
    //        println!("message {:#?}", message);
    //        if let Some(body) = message.bodies.get("") {
    //            let from = message.from.clone().unwrap();
    //            let to = message.to.clone().unwrap();
    //            let body = body.0.to_owned();
    //
    //            let delay = message.payloads
    //                .iter()
    //                .find(|elm| elm.name() == "delay" )
    //                .unwrap();
    //            let stamp = delay.attr("stamp").unwrap();
    //
    //            use chrono::prelude::*;
    //            //use std::time::SystemTime;
    //            //use human_time::ToHumanTimeString;
    //            let stamp: chrono::DateTime<Utc> = stamp.parse().unwrap();
    //
    //            db::add_new_message(&from, &to, &body, &stamp).await;
    //
    //            //let now =
    //            //let diff =  - stamp.timestamp()
    //            //let stamp = chrono::Duration::from(stamp.signed_duration_since(std::time::UNIX_EPOCH));
    //            //let stamp = stamp.timestamp();
    //            //chrono::Duration::from(stamp);
    //
    //            //let stmap SystemTime::from(stamp)
    //            //stamp.to_u
    //
    //            sender.send(BackendEvent::Message {
    //                to: Arc::new(to.to_bare()),
    //                from: Arc::new(from.to_bare()),
    //                body: body.clone(),
    //                by_me: false,
    //            }).await.unwrap();
    //
    //            let id = message.id.clone().unwrap();
    //            let re = recieved_elm(&id);
    //            let dis = displayed_elm(&id);
    //
    //            //println!("{:#?}\n{:#?}", re, dis);
    //
    //            let mut bodies = BTreeMap::new();
    //            bodies.insert("".to_string(), Body(String::from("test11")));
    //
    //            let msg = StanzaMessage {
    //                from: message.to,
    //                to: message.from,
    //                id: message.id,
    //                type_: MessageType::Chat,
    //                bodies,
    //                subjects: BTreeMap::new(),
    //                thread: None,
    //                payloads: vec![re, dis],
    //            };
    //
    //            //message.payloads.get()
    //
    //            //sender.send(BackendEvent::NewMessage {
    //            //    from: from.clone(),
    //            //    to: to.clone(),
    //            //    //message: Message::new_text {content: Content::Text(String::from("sdf")) },
    //            //    message: Message::new_text(body.clone(), false),
    //            //    by_me: false
    //            //    //content:
    //            //}).await.unwrap();
    //
    //            //db.new_text_message(from, to, body, stamp).unwrap();
    //
    //        } else {
    //            println!("{:#?}", message);
    //        }
    //    },
    //    Stanza::Iq(_iq) => {},
    //    Stanza::Presence(_presence) => {},
    //}
}

fn handle_recipt_request() {}

//fn message_recieved<J>(
//    to: J,
//    id: String,
//)
//where
//    J: Into<Option<Jid>>
//{
//    //use xmpp_parsers::message::MessagePayload;
//    //Element::from("sdf");
//    let ss = Element::builder("recieved", "urn:xmpp:receipts")
//        .attr("id", id)
//        .build();
//
//    let ss: Element = r#"
//<message
//    from='kingrichard@royalty.england.lit/throne'
//    id='bi29sg183b4v'
//    to='northumberland@shakespeare.lit/westminster'>
//  <received xmlns='urn:xmpp:receipts' id='richard2-4.1.247'/>
//</message>
//"#.parse().unwrap();
//    //StanzaMessage::new(to).with_payload(MessagePayload::from)
//}

fn handle_payloads(payloads: Vec<Element>, id: String) {
    let mut elms = Vec::new();
    for elm in payloads {
        match elm.name() {
            "request" => {
                let elm = Element::builder("recieved", "urn:xmpp:reciepts").attr("id", id.clone());

                elms.push(elm);
            }

            _ => {}
        };
    }
}

fn recieved_elm(id: &str) -> Element {
    Element::builder("recieved", "urn:xmpp:reciepts:0")
        .attr("id", id)
        .build()
}

fn displayed_elm(id: &str) -> Element {
    Element::builder("displayed", "urn:xmpp:chat-markers:0")
        .attr("id", id)
        .build()
}
