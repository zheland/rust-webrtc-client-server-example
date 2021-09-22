use core::sync::atomic::AtomicBool;
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;

use crate::{ChannelId, ChannelMessage};

#[derive(Clone, Debug)]
pub struct ChannelSender {
    channel_id: ChannelId,
    sender: UnboundedSender<ChannelMessage>,
    has_receiver: Arc<AtomicBool>,
}

impl ChannelSender {
    pub fn new(
        channel_id: ChannelId,
        sender: UnboundedSender<ChannelMessage>,
        has_receiver: Arc<AtomicBool>,
    ) -> Self {
        Self {
            channel_id,
            sender,
            has_receiver,
        }
    }

    pub fn channel_id(&self) -> ChannelId {
        self.channel_id
    }

    pub fn send(&self, message: ChannelMessage) {
        use core::sync::atomic::Ordering;
        if self.has_receiver.load(Ordering::Relaxed) {
            self.sender.send(message).unwrap();
        }
    }
}
