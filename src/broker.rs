use tokio::sync::broadcast;
use crate::message::AgentMessage;

pub struct Broker {
    pub sender: broadcast::Sender<AgentMessage>,
}

impl Broker {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Broker { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AgentMessage> {
        self.sender.subscribe()
    }
}