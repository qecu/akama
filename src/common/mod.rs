pub mod message;
use message::*;

mod contact;
pub use contact::*;

mod account;
pub use account::*;

mod event;
pub use event::*;

mod id;
pub use id::Id;

use std::fmt;

pub type Password = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Status {
    Online(Option<String>), // contains the resource of jid 
    Offline,
    Connecting,
    Away,
    /// Do not disturb
    Dnd,
    /// Extended Away
    Xa,
}


impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let r = match self {
            Status::Online(_) => "Online",
            Status::Offline => "Offline",
            Status::Connecting => "Connecting",
            Status::Away => "Away",
            Status::Dnd => "Dnd",
            Status::Xa => "Extended Away",
        };

        f.write_str(r)
    }
}
