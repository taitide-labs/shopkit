use shopkit_contracts::events::IntegrationEvent;
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct EventBus {
    sender: broadcast::Sender<IntegrationEvent>,
}

impl EventBus {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (sender, _receiver) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: IntegrationEvent) {
        let _listener_count = self.sender.send(event);
    }

    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<IntegrationEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}
