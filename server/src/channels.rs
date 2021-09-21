use core::sync::atomic::AtomicU32;
use std::collections::VecDeque;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{ChannelId, ChannelMessage, ChannelReceiver, ChannelSender};

#[derive(Debug)]
pub struct Channels {
    senders: VecDeque<UnboundedSender<ChannelMessage>>,
    receivers: VecDeque<UnboundedReceiver<ChannelMessage>>,
    next_sender_channel_id: AtomicU32,
    next_receiver_channel_id: AtomicU32,
}

impl Channels {
    pub fn new() -> Self {
        let senders = VecDeque::new();
        let receivers = VecDeque::new();
        let next_sender_channel_id = AtomicU32::new(0);
        let next_receiver_channel_id = AtomicU32::new(0);

        Self {
            next_sender_channel_id,
            next_receiver_channel_id,
            senders,
            receivers,
        }
    }

    pub fn sender(&mut self) -> ChannelSender {
        use core::sync::atomic::Ordering;
        use tokio::sync::mpsc::unbounded_channel;

        let channel_id = self.next_sender_channel_id.fetch_add(1, Ordering::Relaxed);
        let sender = self.senders.pop_front().unwrap_or_else(|| {
            let (sender, receiver) = unbounded_channel();
            self.receivers.push_back(receiver);
            sender
        });

        ChannelSender::new(ChannelId(channel_id), sender)
    }

    pub fn receiver(&mut self) -> ChannelReceiver {
        use core::sync::atomic::Ordering;
        use tokio::sync::mpsc::unbounded_channel;

        let channel_id = self
            .next_receiver_channel_id
            .fetch_add(1, Ordering::Relaxed);
        let receiver = self.receivers.pop_front().unwrap_or_else(|| {
            let (sender, receiver) = unbounded_channel();
            self.senders.push_back(sender);
            receiver
        });

        ChannelReceiver::new(ChannelId(channel_id), receiver)
    }
}
