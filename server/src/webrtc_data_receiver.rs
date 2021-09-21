use core::fmt;
use std::sync::Arc;

use webrtc::data::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data::data_channel::RTCDataChannel;

pub struct WebRtcDataReceiver {
    data_channel: Arc<RTCDataChannel>,
}

impl WebRtcDataReceiver {
    pub async fn new(data_channel: Arc<RTCDataChannel>) -> Arc<Self> {
        let receiver = Arc::new(Self { data_channel });

        receiver.init().await;

        receiver
    }

    async fn init(self: &Arc<Self>) {
        use crate::WeakAsyncCallback;

        self.data_channel
            .on_message(Box::with_weak_async_callback(self, Self::on_message))
            .await;
    }

    pub async fn on_message(self: Arc<Self>, msg: DataChannelMessage) {
        let data = &msg.data[1..];
        let string = String::from_utf8_lossy(data);

        log::debug!(
            "Message from DataChannel '{}': '{}'\n",
            self.data_channel.label(),
            string
        );
    }
}

impl fmt::Debug for WebRtcDataReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebRtcDataReceiver").finish_non_exhaustive()
    }
}
