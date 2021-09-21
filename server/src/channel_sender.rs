use tokio::sync::mpsc::UnboundedSender;

use crate::{ChannelId, ChannelMessage};

#[derive(Debug)]
pub struct ChannelSender {
    channel_id: ChannelId,
    sender: UnboundedSender<ChannelMessage>,
}

impl ChannelSender {
    pub fn new(channel_id: ChannelId, sender: UnboundedSender<ChannelMessage>) -> Self {
        Self { channel_id, sender }
    }

    pub fn channel_id(&self) -> ChannelId {
        self.channel_id
    }

    pub fn _sender(&self) -> &UnboundedSender<ChannelMessage> {
        &self.sender
    }
}
