use std::collections::{BTreeMap, HashMap};
use chrono::Utc;
use futures::sink::SinkExt;
use futures::stream::{Map, SelectAll, StreamExt, SplitSink, SplitStream};
use tokio_xmpp::{
    Client, 
    Event as TokioXmppEvent, 
    Stanza, jid::Jid,
    connect::starttls::StartTlsClient as XmppTlsClient,
    parsers::{
        minidom::Element,
        message::{Body, Message as StanzaMessage, MessageType},
        presence::{Presence, Show as PresenceShow, Type as PresenceType},
    }
};
use async_channel::{Receiver, Sender};
use crate::common::{Account, Id, message::Message, AccountId, Status, Contact, BackendEvent, UiEvent};
mod db;
mod stanza;


pub struct Backend {
    tx: Sender<BackendEvent>,
    rx: Receiver<UiEvent>,
    //client_st: SelectAll<Box<dyn IntoStream>>,
    client_sk: HashMap<AccountId, SplitSink<XmppTlsClient, Stanza>>,
    //client_sk: HashMap<Id, SplitStream<Client<XmppTlsClient>>>
}

impl Backend {
    pub async fn run(tx: Sender<BackendEvent>, rx: Receiver<UiEvent>) {
        db::setup().await.unwrap();
        let accounts = db::get_accounts().await;

        //let mut streams = SelectAll::new();
        //let mut sinks = HashMap::new();

        let mut streams = SelectAll::new();
        let sinks = HashMap::new();

        let mut backend = Self {
            tx: tx.clone(),
            rx: rx.clone(),
            client_sk: sinks,
        };

        for (jid, password) in accounts {
            log::debug!("account: {}", jid);

            let xmpp_client = Client::new((*jid).clone(), password);
            let (client_sink, client_stream) = xmpp_client.split();

            streams.push(stream_to_jid_stream(client_stream, jid.clone()));
            backend.client_sk.insert(jid.clone(), client_sink);

            tx.send(BackendEvent::Account(
                jid.clone(),
                Account::new((**jid).clone(), Status::Connecting),
            ))
            .await
            .unwrap();

            backend.send_contacts(&jid).await.unwrap();
        }
        
        use crate::common::Id;
        use std::sync::Arc;
        
        loop {

            tokio::select! {
                Some((jid, event)) = streams.next() => {
                    backend.handle_xmpp_event(&jid, event).await;
                },
                ui_event = backend.rx.recv() => {
                    let ui_event = ui_event.unwrap();

                    match ui_event {
                        UiEvent::NewXmppClient { jid, password, client } => {
                            // use crate::core::Id;
                            let resource = jid.resource();
                            let id = Id(Arc::new(jid.to_bare()));
                            db::add_account(&id, resource, &password).await;
                            let (sink, stream) = client.0.split();
                            backend.client_sk.insert(id.clone(), sink);
                            streams.push(stream_to_jid_stream(stream, id.clone()));
                        }
                        other => backend.handle_ui_event(other).await,
                    }
                    //handle_ui_event(&sender, ui_event, &mut sinks).await;
                }
            }
        }
    }

    async fn handle_xmpp_event(&mut self, id: &Id, event: TokioXmppEvent) {
        match event {
            TokioXmppEvent::Online { bound_jid, resumed: _ } => {
                log::info!("account online: {}", bound_jid);
                let presence = make_presence();
                self.client_sk
                    .get_mut(id)
                    .unwrap()
                    .send(presence.into())
                    .await
                    .unwrap();
                
                let resource = bound_jid.resource().map(|r| r.to_string());
                self.tx
                    .send(BackendEvent::AccountStatusUpdate(id.clone(), Status::Online(resource)))
                    .await
                    .unwrap();
            }
            TokioXmppEvent::Disconnected(error) => {
                //WARN: Hanlde this
                log::error!("disconnect: {}", error);
            }
            TokioXmppEvent::Stanza(stanza) => {
                match stanza {
                    Stanza::Iq(iq) => self.handle_iq(iq),
                    Stanza::Message(message) => self.handle_message(message).await,
                    Stanza::Presence(presence) => self.handle_presence(presence),
                };
            }
        }
    }

    async fn handle_ui_event(&mut self, event: UiEvent) {
        match event {
            UiEvent::NewText { from, to, content } => {
                let from_ = (*from).clone();
                let to_ = (*to).clone();

                let msg = new_text_stanza(from_, to_, content.clone());

                let sink = self.client_sk.get_mut(&from).unwrap();
                sink.send(Stanza::Message(msg)).await.unwrap();

                let stamp = Utc::now();

                db::add_text_message(&from, to.as_ref(), content.as_str(), &stamp).await;

                self.tx
                    .send(BackendEvent::Message {
                        from,
                        to,
                        body: content,
                        by_me: true,
                        timestamp: stamp,
                        id: None,
                    })
                    .await
                    .unwrap();
            }
            UiEvent::NewContact(id, c_id) => {
                db::add_contact(&id, &c_id).await;
            }
            UiEvent::NewXmppClient { jid, password, client } => {

            }

            //UiEvent::DisconnectAccount(_jid) => {
            //    //let sink = client_sinks.get(&jid).unwrap();
            //    // sink.close();
            //
            //    //let sender = self.channels.get(&jid).unwrap();
            //    //sender.send(Command::Disconnect).await.unwrap();
            //}
            //UiEvent::NewTextMessage { from, to, content } => {
            //    //let sender = self.channels.get(&from).unwrap();
            //    //sender.send(Comm)
            //}
            other => {
                log::warn!("Unhandled Ui Event: {:?}", other);
            }
        };
    }

    async fn send_contacts(&self, jid: &AccountId) -> anyhow::Result<()> {
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
            self.tx.send(BackendEvent::Contacts(contactsss)).await?;
        }

        Ok(())
    }

    fn tx(&self) -> Sender<BackendEvent> {
        self.tx.clone()
    }

    fn rx(&self) -> Receiver<UiEvent> {
        self.rx.clone()
    }
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
