use core::sync::atomic::AtomicBool;
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

use crate::{ChannelId, ChannelMessage};

#[derive(Debug)]
pub struct ChannelReceiver {
    channel_id: ChannelId,
    receiver: Mutex<UnboundedReceiver<ChannelMessage>>,
    has_receiver: Arc<AtomicBool>,
}

impl ChannelReceiver {
    pub fn new(
        channel_id: ChannelId,
        receiver: UnboundedReceiver<ChannelMessage>,
        has_receiver: Arc<AtomicBool>,
    ) -> Self {
        let receiver = Mutex::new(receiver);

        Self {
            channel_id,
            receiver,
            has_receiver,
        }
    }

    pub fn channel_id(&self) -> ChannelId {
        self.channel_id
    }

    pub async fn recv(&self) -> Option<ChannelMessage> {
        use core::sync::atomic::Ordering;
        self.has_receiver.store(true, Ordering::Relaxed);
        self.receiver.lock().await.recv().await
    }
}
