use core::fmt;
use std::sync::Arc;

use protocol::{IceCandidate, ServerSenderMessage, SessionDescription};
use tokio::sync::Mutex;
use webrtc::data::data_channel::RTCDataChannel;
use webrtc::media::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::peer::ice::ice_candidate::RTCIceCandidate;
use webrtc::peer::peer_connection::RTCPeerConnection;
use webrtc::peer::peer_connection_state::RTCPeerConnectionState;

use crate::{ChannelId, ChannelReceiver, WebRtcApi, WebSocketSender};

pub struct WebRtcSender {
    api: Arc<WebRtcApi>,
    channel_receiver: ChannelReceiver,
    peer_connection: RTCPeerConnection,
    websocket_sender: Mutex<WebSocketSender<ServerSenderMessage>>,
    delayed_icecandidates: Mutex<Vec<IceCandidate>>,
    data_channel: Arc<RTCDataChannel>,
    media_track: Arc<TrackLocalStaticRTP>,
}

impl WebRtcSender {
    pub async fn new(
        api: Arc<WebRtcApi>,
        channel_receiver: ChannelReceiver,
        websocket_sender: WebSocketSender<ServerSenderMessage>,
    ) -> Arc<Self> {
        use webrtc::api::media_engine::MIME_TYPE_VP8;
        use webrtc::media::rtp::rtp_codec::RTCRtpCodecCapability;
        use webrtc::media::track::track_local::TrackLocal;

        let channel_receiver = channel_receiver;
        let peer_connection = api.new_peer_connection().await;
        let websocket_sender = Mutex::new(websocket_sender);
        let delayed_icecandidates = Mutex::new(Vec::new());

        let data_channel = peer_connection
            .create_data_channel("data", None)
            .await
            .unwrap();
        let media_track = Arc::new(TrackLocalStaticRTP::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_VP8.to_owned(),
                ..Default::default()
            },
            "video".to_owned(),
            "webrtc-rs".to_owned(),
        ));

        #[allow(trivial_casts)] // false positive
        let media_track_ref = Arc::clone(&media_track) as Arc<dyn TrackLocal + Sync + Send>;
        let _ = peer_connection.add_track(media_track_ref).await.unwrap();

        let receiver = Arc::new(Self {
            api,
            channel_receiver,
            websocket_sender,
            peer_connection,
            delayed_icecandidates,
            data_channel,
            media_track,
        });

        receiver.init().await;

        receiver
    }

    async fn init(self: &Arc<Self>) {
        self.init_handlers().await;
        self.send_offer().await;
        self.spawn_thread();
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
    }

    async fn send_offer(self: &Arc<Self>) {
        let offer = self.peer_connection.create_offer(None).await.unwrap();

        let offer_sdp = offer.serde.sdp.clone();
        self.peer_connection
            .set_local_description(offer)
            .await
            .unwrap();

        self.websocket_sender
            .lock()
            .await
            .send(ServerSenderMessage::Offer(SessionDescription(offer_sdp)))
            .await;
    }

    pub async fn on_answer(self: &Arc<Self>, sdp: SessionDescription) {
        use core::mem::take;
        use webrtc::peer::sdp::sdp_type::RTCSdpType;
        use webrtc::peer::sdp::session_description::{
            RTCSessionDescription, RTCSessionDescriptionSerde,
        };

        let mut asnwer = RTCSessionDescription::default();
        asnwer.serde = RTCSessionDescriptionSerde {
            sdp_type: RTCSdpType::Answer,
            sdp: sdp.0,
        };

        self.peer_connection
            .set_remote_description(asnwer)
            .await
            .unwrap();

        let mut icecandidates = self.delayed_icecandidates.lock().await;
        let icecandidates: Vec<_> = take(&mut icecandidates);
        for candidate in icecandidates {
            self.on_remote_icecandidate(candidate).await;
        }
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
            "channel {}: sender peer connection state has changed: {}",
            self.channel_id(),
            state
        );
    }
    async fn on_local_icecandidate(self: Arc<Self>, ice_candidate: Option<RTCIceCandidate>) {
        crate::send_local_icecandidate(
            &self.websocket_sender,
            ice_candidate,
            ServerSenderMessage::IceCandidate,
            ServerSenderMessage::AllIceCandidatesSent,
        )
        .await;
    }

    pub fn channel_id(&self) -> ChannelId {
        self.channel_receiver.channel_id()
    }

    fn spawn_thread(self: &Arc<Self>) {
        use tokio::spawn;
        use tokio::task::JoinHandle;

        let self_arc = Arc::clone(&self);
        let _: JoinHandle<()> = spawn(async move { self_arc.thread().await });
    }

    async fn thread(self: &Arc<Self>) {
        use crate::ChannelMessage;
        use bytes::Bytes;
        use rtp::packet::Packet;
        use webrtc::data::data_channel::data_channel_state::RTCDataChannelState;
        use webrtc::media::track::track_local::TrackLocalWriter;
        use webrtc_util::marshal::Unmarshal;

        while let Some(message) = self.channel_receiver.recv().await {
            match message {
                ChannelMessage::Data(data) => {
                    if self.data_channel.ready_state() == RTCDataChannelState::Open {
                        let _: usize = self
                            .data_channel
                            .send(&Bytes::copy_from_slice(&data))
                            .await
                            .unwrap();
                    }
                }
                ChannelMessage::Media(data) => {
                    let mut buf = data.as_slice();
                    let rtp = Packet::unmarshal(&mut buf).unwrap();
                    let _: usize = self.media_track.write_rtp(&rtp).await.unwrap();
                }
            }
        }
    }
}

impl fmt::Debug for WebRtcSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebRtcSender")
            .field("api", &self.api)
            .field("channel_receiver", &self.channel_receiver)
            .field("websocket_sender", &self.websocket_sender)
            .finish_non_exhaustive()
    }
}
