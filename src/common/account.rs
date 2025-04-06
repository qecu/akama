use super::Status;
use tokio_xmpp::parsers::jid::Jid;
use super::Id;

pub type AccountId = Id;

#[derive(Debug, Clone)]
pub struct Account {
    id: AccountId,
    resource: Option<String>,
    status: Status,
}

impl Account {
    pub fn new(jid: Jid, status: Status) -> Self {
        let resource = jid.resource().map(|r| r.to_string());
        let id =Id::new(jid);

        Self { id, resource, status }
    }
    
    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn resource(&self) -> &Option<String> {
        &self.resource
    }

    pub fn set_jid_resource(&mut self, resource: String) {
        self.resource = Some(resource);
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status
    }
}
