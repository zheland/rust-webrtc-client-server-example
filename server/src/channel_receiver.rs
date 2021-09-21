use tokio::sync::mpsc::UnboundedReceiver;

use crate::{ChannelId, ChannelMessage};

#[derive(Debug)]
pub struct ChannelReceiver {
    channel_id: ChannelId,
    receiver: UnboundedReceiver<ChannelMessage>,
}

impl ChannelReceiver {
    pub fn new(channel_id: ChannelId, receiver: UnboundedReceiver<ChannelMessage>) -> Self {
        Self {
            channel_id,
            receiver,
        }
    }

    pub fn _channel_id(&self) -> ChannelId {
        self.channel_id
    }

    pub fn _receiver(&self) -> &UnboundedReceiver<ChannelMessage> {
        &self.receiver
    }
}
