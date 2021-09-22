use core::fmt;
use core::sync::atomic::AtomicBool;
use std::sync::Arc;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{ChannelMessage, ChannelReceiver, ChannelSender};

#[derive(Debug)]
pub struct Channel {
    channel_id: ChannelId,
    sender: UnboundedSender<ChannelMessage>,
    receiver: UnboundedReceiver<ChannelMessage>,
    has_receiver: Arc<AtomicBool>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ChannelId(pub u32);

impl Channel {
    pub fn new(channel_id: ChannelId) -> Self {
        use tokio::sync::mpsc::unbounded_channel;
        let (sender, receiver) = unbounded_channel();
        let has_receiver = Arc::new(AtomicBool::new(false));

        Self {
            channel_id,
            sender,
            receiver,
            has_receiver,
        }
    }

    pub fn split(self) -> (ChannelSender, ChannelReceiver) {
        (
            ChannelSender::new(self.channel_id, self.sender, Arc::clone(&self.has_receiver)),
            ChannelReceiver::new(self.channel_id, self.receiver, self.has_receiver),
        )
    }
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
