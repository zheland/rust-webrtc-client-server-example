use core::fmt;
use std::sync::Arc;

use protocol::{IceCandidate, ServerReceiverMessage, SessionDescription};

use crate::{ChannelReceiver, WebRtcApi, WebSocketSender};

pub struct WebRtcSender {}

impl WebRtcSender {
    pub async fn new(
        _api: Arc<WebRtcApi>,
        _channel_receiver: ChannelReceiver,
        _websocket_sender: WebSocketSender<ServerReceiverMessage>,
    ) -> Arc<Self> {
        todo!()
    }

    pub async fn on_answer(self: &Arc<Self>, _offer: SessionDescription) {
        todo!()
    }

    pub async fn on_remote_icecandidate(self: &Arc<Self>, _offer: IceCandidate) {
        todo!()
    }

    pub async fn on_all_remote_icecandidates_sent(self: &Arc<Self>) {
        todo!()
    }
}

impl fmt::Debug for WebRtcSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebRtcSender").finish_non_exhaustive()
    }
}
