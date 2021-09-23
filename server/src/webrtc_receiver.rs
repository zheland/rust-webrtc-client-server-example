use core::fmt;
use std::sync::Arc;

use protocol::{IceCandidate, ServerReceiverMessage, SessionDescription};
use tokio::sync::{Mutex, RwLock};
use webrtc::data::data_channel::RTCDataChannel;
use webrtc::media::rtp::rtp_receiver::RTCRtpReceiver;
use webrtc::media::track::track_remote::TrackRemote;
use webrtc::peer::ice::ice_candidate::RTCIceCandidate;
use webrtc::peer::peer_connection::RTCPeerConnection;
use webrtc::peer::peer_connection_state::RTCPeerConnectionState;

use crate::{
    ChannelId, ChannelSender, WebRtcApi, WebRtcDataReceiver, WebRtcMediaReceiver, WebSocketSender,
};

pub struct WebRtcReceiver {
    api: Arc<WebRtcApi>,
    channel_sender: ChannelSender,
    peer_connection: RTCPeerConnection,
    websocket_sender: Mutex<WebSocketSender<ServerReceiverMessage>>,
    data_receivers: RwLock<Vec<Arc<WebRtcDataReceiver>>>,
    media_receivers: RwLock<Vec<Arc<WebRtcMediaReceiver>>>,
    delayed_icecandidates: Mutex<Vec<IceCandidate>>,
}

impl WebRtcReceiver {
    pub async fn new(
        api: Arc<WebRtcApi>,
        channel_sender: ChannelSender,
        websocket_sender: WebSocketSender<ServerReceiverMessage>,
    ) -> Arc<Self> {
        let peer_connection = api.new_peer_connection().await;
        let websocket_sender = Mutex::new(websocket_sender);
        let data_receivers = RwLock::new(Vec::new());
        let media_receivers = RwLock::new(Vec::new());
        let delayed_icecandidates = Mutex::new(Vec::new());

        let receiver = Arc::new(Self {
            api,
            channel_sender,
            websocket_sender,
            peer_connection,
            data_receivers,
            media_receivers,
            delayed_icecandidates,
        });

        receiver.init().await;

        receiver
    }

    async fn init(self: &Arc<Self>) {
        self.init_handlers().await;
    }

    async fn init_handlers(self: &Arc<Self>) {
        use crate::WeakAsyncCallback;

        self.peer_connection
            .on_peer_connection_state_change(Box::with_weak_async_callback(
                self,
                Self::on_peer_connection_state_change,
            ))
            .await;

        self.peer_connection
            .on_ice_candidate(Box::with_weak_async_callback(
                self,
                Self::on_local_icecandidate,
            ))
            .await;

        self.peer_connection
            .on_data_channel(Box::with_weak_async_callback(self, Self::on_data_channel))
            .await;

        self.peer_connection
            .on_track(Box::with_weak_async_callback(self, Self::on_track))
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

        self.send_answer().await;

        let mut icecandidates = self.delayed_icecandidates.lock().await;
        let icecandidates: Vec<_> = take(&mut icecandidates);
        for candidate in icecandidates {
            self.on_remote_icecandidate(candidate).await;
        }
    }

    async fn send_answer(self: &Arc<Self>) {
        let answer = self.peer_connection.create_answer(None).await.unwrap();

        let answer_sdp = answer.serde.sdp.clone();
        self.peer_connection
            .set_local_description(answer)
            .await
            .unwrap();

        self.websocket_sender
            .lock()
            .await
            .send(ServerReceiverMessage::Answer(SessionDescription(
                answer_sdp,
            )))
            .await;
    }

    pub async fn on_remote_icecandidate(self: &Arc<Self>, ice_candidate: IceCandidate) {
        crate::add_remote_icecandidate(
            &self.peer_connection,
            ice_candidate,
            &self.delayed_icecandidates,
        )
        .await;
    }

    pub async fn on_all_remote_icecandidates_sent(self: &Arc<Self>) {}

    async fn on_peer_connection_state_change(self: Arc<Self>, state: RTCPeerConnectionState) {
        log::info!(
            "channel {}: receiver peer connection state has changed: {}",
            self.channel_id(),
            state
        );
    }

    async fn on_local_icecandidate(self: Arc<Self>, ice_candidate: Option<RTCIceCandidate>) {
        crate::send_local_icecandidate(
            &self.websocket_sender,
            ice_candidate,
            ServerReceiverMessage::IceCandidate,
            ServerReceiverMessage::AllIceCandidatesSent,
        )
        .await;
    }

    fn channel_id(&self) -> ChannelId {
        self.channel_sender.channel_id()
    }

    async fn on_data_channel(self: Arc<Self>, data_channel: Arc<RTCDataChannel>) {
        let data_receiver =
            WebRtcDataReceiver::new(self.channel_sender.clone(), data_channel).await;
        self.data_receivers.write().await.push(data_receiver);
    }

    async fn on_track(
        self: Arc<Self>,
        track: Option<Arc<TrackRemote>>,
        receiver: Option<Arc<RTCRtpReceiver>>,
    ) {
        if let Some(track) = track {
            let media_receiver =
                WebRtcMediaReceiver::new(self.channel_sender.clone(), track, receiver).await;
            self.media_receivers.write().await.push(media_receiver);
        }
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
