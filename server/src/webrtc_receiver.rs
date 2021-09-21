use core::fmt;
use std::sync::Arc;

use protocol::{IceCandidate, ServerSenderMessage, SessionDescription};
use tokio::sync::{Mutex, RwLock};
use webrtc::data::data_channel::RTCDataChannel;
use webrtc::peer::ice::ice_candidate::RTCIceCandidate;
use webrtc::peer::peer_connection::RTCPeerConnection;
use webrtc::peer::peer_connection_state::RTCPeerConnectionState;

use crate::{ChannelId, ChannelSender, WebRtcApi, WebRtcDataReceiver, WebSocketSender};

pub struct WebRtcReceiver {
    api: Arc<WebRtcApi>,
    channel_sender: ChannelSender,
    peer_connection: RTCPeerConnection,
    websocket_sender: Mutex<WebSocketSender<ServerSenderMessage>>,
    data_receivers: RwLock<Vec<Arc<WebRtcDataReceiver>>>,
    delayed_icecandidates: Mutex<Vec<IceCandidate>>,
}

impl WebRtcReceiver {
    pub async fn new(
        api: Arc<WebRtcApi>,
        channel_sender: ChannelSender,
        websocket_sender: WebSocketSender<ServerSenderMessage>,
    ) -> Arc<Self> {
        let peer_connection = api.new_peer_connection().await;
        let data_receivers = RwLock::new(Vec::new());
        let websocket_sender = Mutex::new(websocket_sender);
        let delayed_icecandidates = Mutex::new(Vec::new());

        let receiver = Arc::new(Self {
            api,
            channel_sender,
            websocket_sender,
            peer_connection,
            data_receivers,
            delayed_icecandidates,
        });

        receiver.init().await;

        receiver
    }

    async fn init(self: &Arc<Self>) {
        use crate::WeakAsyncCallback;

        self.peer_connection
            .on_peer_connection_state_change(Box::with_weak_async_callback(
                self,
                Self::on_peer_connection_state_change,
            ))
            .await;

        self.peer_connection
            .on_data_channel(Box::with_weak_async_callback(self, Self::on_data_channel))
            .await;

        self.peer_connection
            .on_ice_candidate(Box::with_weak_async_callback(
                self,
                Self::on_local_icecandidate,
            ))
            .await;
    }

    pub async fn on_offer(self: &Arc<Self>, sdp: SessionDescription) {
        use core::mem::take;
        use webrtc::peer::sdp::sdp_type::RTCSdpType;
        use webrtc::peer::sdp::session_description::{
            RTCSessionDescription, RTCSessionDescriptionSerde,
        };

        let mut offer = RTCSessionDescription::default();
        offer.serde = RTCSessionDescriptionSerde {
            sdp_type: RTCSdpType::Offer,
            sdp: sdp.0,
        };

        self.peer_connection
            .set_remote_description(offer)
            .await
            .unwrap();
        let answer = self.peer_connection.create_answer(None).await.unwrap();

        let _ = self.peer_connection.gathering_complete_promise().await;

        let answer_sdp = answer.serde.sdp.clone();
        self.peer_connection
            .set_local_description(answer)
            .await
            .unwrap();

        self.websocket_sender
            .lock()
            .await
            .send(ServerSenderMessage::Answer(SessionDescription(answer_sdp)))
            .await;

        let mut icecandidates = self.delayed_icecandidates.lock().await;
        let icecandidates: Vec<_> = take(&mut icecandidates);

        for candidate in icecandidates {
            self.on_remote_icecandidate(candidate).await;
        }
    }

    pub async fn on_remote_icecandidate(self: &Arc<Self>, candidate: IceCandidate) {
        use webrtc::peer::ice::ice_candidate::RTCIceCandidateInit;

        if self.peer_connection.remote_description().await.is_some() {
            let candidate = RTCIceCandidateInit {
                candidate: candidate.candidate,
                sdp_mid: candidate.sdp_mid.unwrap_or_else(|| String::new()),
                sdp_mline_index: candidate.sdp_mline_index.unwrap_or(0),
                username_fragment: candidate.username_fragment.unwrap_or_else(|| String::new()),
            };

            self.peer_connection
                .add_ice_candidate(candidate)
                .await
                .unwrap();
        } else {
            self.delayed_icecandidates.lock().await.push(candidate);
        }
    }

    pub async fn on_all_remote_icecandidates_sent(self: &Arc<Self>) {}

    pub async fn on_peer_connection_state_change(self: Arc<Self>, state: RTCPeerConnectionState) {
        log::info!(
            "channel {}: receiver peer connection state has changed: {}",
            self.channel_id(),
            state
        );
    }

    pub async fn on_data_channel(self: Arc<Self>, data_channel: Arc<RTCDataChannel>) {
        let data_receiver = WebRtcDataReceiver::new(data_channel).await;
        self.data_receivers.write().await.push(data_receiver);
    }

    pub async fn on_local_icecandidate(self: Arc<Self>, ice_candidate: Option<RTCIceCandidate>) {
        if let Some(ice_candidate) = ice_candidate {
            let json = ice_candidate.to_json().await.unwrap();

            self.websocket_sender
                .lock()
                .await
                .send(ServerSenderMessage::IceCandidate(IceCandidate {
                    candidate: json.candidate,
                    sdp_mid: Some(json.sdp_mid),
                    sdp_mline_index: Some(json.sdp_mline_index),
                    username_fragment: Some(json.username_fragment),
                }))
                .await
        } else {
            self.websocket_sender
                .lock()
                .await
                .send(ServerSenderMessage::AllIceCandidatesSent)
                .await
        }
    }

    pub fn channel_id(&self) -> ChannelId {
        self.channel_sender.channel_id()
    }
}

impl fmt::Debug for WebRtcReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebRtcReceiver")
            .field("api", &self.api)
            .field("channel_sender", &self.channel_sender)
            .field("websocket_sender", &self.websocket_sender)
            .finish_non_exhaustive()
    }
}
