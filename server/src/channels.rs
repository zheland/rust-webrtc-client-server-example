use core::sync::atomic::AtomicU32;
use std::collections::VecDeque;

use crate::{Channel, ChannelId, ChannelReceiver, ChannelSender};

#[derive(Debug)]
pub struct Channels {
    senders: VecDeque<ChannelSender>,
    receivers: VecDeque<ChannelReceiver>,
    next_channel_id: AtomicU32,
}

impl Channels {
    pub fn new() -> Self {
        let senders = VecDeque::new();
        let receivers = VecDeque::new();
        let next_channel_id = AtomicU32::new(0);

        Self {
            senders,
            receivers,
            next_channel_id,
        }
    }

    pub fn sender(&mut self) -> ChannelSender {
        use core::sync::atomic::Ordering;

        self.senders.pop_front().unwrap_or_else(|| {
            let channel_id = ChannelId(self.next_channel_id.fetch_add(1, Ordering::Relaxed));
            let (sender, receiver) = Channel::new(channel_id).split();
            self.receivers.push_back(receiver);
            sender
        })
    }

    pub fn receiver(&mut self) -> ChannelReceiver {
        use core::sync::atomic::Ordering;

        self.receivers.pop_front().unwrap_or_else(|| {
            let channel_id = ChannelId(self.next_channel_id.fetch_add(1, Ordering::Relaxed));
            let (sender, receiver) = Channel::new(channel_id).split();
            self.senders.push_back(sender);
            receiver
        })
    }
}
