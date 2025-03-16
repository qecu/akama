pub mod message;
use message::*;

pub mod contact;
pub use contact::*;

pub mod account;
pub use account::*;

pub mod event;
use event::*;

pub mod id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Online,
    Offline,
    Connecting,
}

pub type Password = String;
