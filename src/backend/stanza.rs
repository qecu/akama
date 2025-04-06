use chrono::Utc;
use tokio_xmpp::parsers::iq::Iq;
use tokio_xmpp::parsers::message::Message;
use tokio_xmpp::parsers::presence::Presence;
use super::Backend;
use crate::backend::db;
use crate::common::BackendEvent;


impl Backend {
    pub(super) fn handle_iq(&mut self, iq: Iq) {
        // WARN: handle this
        println!("iq: {:#?}", iq);
    }

    pub(super) fn handle_presence(&mut self, presence: Presence) {
        // WARN: handle this
        println!("iq: {:#?}", presence);
        //println!(")
    }

    pub(super) async fn handle_message(&mut self, message: Message) {
        println!("message {:#?}", message);
        //message.with_body

        if let Some(body) = message.bodies.get("") {
            let from = message.from.clone().unwrap();
            let to = message.to.clone().unwrap();
            let body = body.0.to_owned();
            let id = message.id;

            db::add_text_message(&from.to_bare(), &to.to_bare(), body.as_str(), &Utc::now()).await;

            //let now = time::now();
            //let relative = time::timestamp_to_relative_time(now);

            self.tx
                .send(BackendEvent::Message {
                    to: to.into(),
                    from: from.into(),
                    body: body.clone(),
                    by_me: false,
                    timestamp: chrono::Utc::now(),
                    id: id.clone(),
                })
                .await
                .unwrap();

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

            //let id = message.id.clone().unwrap();
            //let re = recieved_elm(&id);
            //let dis = displayed_elm(&id);

            //println!("my {:#?}\nmy {:#?}", re, dis);

            //let mut bodies = BTreeMap::new();
            //bodies.insert("".to_string(), Body(String::from("test11")));
            //
            //let msg = Message {
            //    from: message.to,
            //    to: message.from,
            //    id: Some(Uuid::new_v4().to_string()),
            //    type_: MessageType::Chat,
            //    bodies,
            //    subjects: BTreeMap::new(),
            //    thread: None,
            //    //payloads: vec![re, dis],
            //    payloads: Vec::new(),
            //};

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
}
