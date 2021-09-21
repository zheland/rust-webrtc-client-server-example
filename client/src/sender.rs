use core::cell::RefCell;

use async_std::sync::Arc;
use protocol::{IceCandidate, SessionDescription};
use web_sys::{
    Event, HtmlTextAreaElement, HtmlVideoElement, MediaStream, MessageEvent, RtcDataChannel,
    RtcPeerConnection, RtcPeerConnectionIceEvent, WebSocket,
};

use crate::ClosureCell1;

#[derive(Debug)]
pub struct Sender {
    websocket: WebSocket,
    webrtc: RtcPeerConnection,
    data_channel: RtcDataChannel,
    media_stream: MediaStream,
    video: HtmlVideoElement,
    text: HtmlTextAreaElement,
    message_handler: ClosureCell1<MessageEvent>,
    icecandidate_handler: ClosureCell1<RtcPeerConnectionIceEvent>,
    negotiationneeded_handler: ClosureCell1<Event>,
    iceconnectionstatechange_handler: ClosureCell1<Event>,
    icegatheringstatechange_handler: ClosureCell1<Event>,
    signalingstatechange_handler: ClosureCell1<Event>,
    input_handler: ClosureCell1<Event>,
}

impl Sender {
    pub async fn new(addr: String) -> Arc<Self> {
        use crate::{body, navigator, ElementExt, RtcConfigurationExt};
        use js_sys::Promise;
        use wasm_bindgen::{JsCast, JsValue};
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{
            BinaryType, MediaStreamConstraints, MediaStreamTrack, RtcConfiguration, RtcRtpSender,
        };

        let mut constraints = MediaStreamConstraints::new();
        let _: &mut _ = constraints.video(&JsValue::TRUE);
        let _: &mut _ = constraints.audio(&JsValue::TRUE);

        let navigator = navigator();
        let media_devices = navigator.media_devices().unwrap();
        let media_stream_promise = media_devices
            .get_user_media_with_constraints(&constraints)
            .unwrap();

        let media_stream: MediaStream = JsFuture::from(media_stream_promise)
            .await
            .unwrap()
            .dyn_into()
            .unwrap();

        let video: HtmlVideoElement = body().add_child("video");
        video.set_autoplay(true);
        video.set_muted(true);
        let _: Option<_> = video.set_attribute("playsinline", "").ok();

        let text: HtmlTextAreaElement = body().add_child("textarea");

        video.set_src_object(Some(&media_stream));

        let conf = RtcConfiguration::new().with_google_stun_server();
        let webrtc = RtcPeerConnection::new_with_configuration(&conf).unwrap();

        for track in media_stream.get_tracks().iter() {
            let track: MediaStreamTrack = track.dyn_into().unwrap();
            let _: RtcRtpSender = webrtc.add_track_0(&track, &media_stream);
        }
        let data_channel = webrtc.create_data_channel("default");

        let websocket = WebSocket::new(addr.as_ref()).unwrap();
        websocket.set_binary_type(BinaryType::Arraybuffer);

        let web_socket_opened = Promise::new(&mut |resolve, reject| {
            websocket.set_onopen(Some(&resolve));
            websocket.set_onerror(Some(&reject));
        });
        let _: JsValue = JsFuture::from(web_socket_opened).await.unwrap();

        let sender = Arc::new(Self {
            websocket,
            webrtc,
            media_stream,
            data_channel,
            video,
            text,
            message_handler: RefCell::new(None),
            icecandidate_handler: RefCell::new(None),
            negotiationneeded_handler: RefCell::new(None),
            iceconnectionstatechange_handler: RefCell::new(None),
            icegatheringstatechange_handler: RefCell::new(None),
            signalingstatechange_handler: RefCell::new(None),
            input_handler: RefCell::new(None),
        });

        sender.init().await;

        sender
    }

    async fn init(self: &Arc<Self>) {
        use crate::init_weak_callback;
        use web_sys::HtmlElement;

        self.start_receiver().await;

        init_weak_callback(
            &self,
            Self::on_message,
            &self.message_handler,
            WebSocket::set_onmessage,
            &self.websocket,
        );

        init_weak_callback(
            &self,
            Self::on_icecandidate,
            &self.icecandidate_handler,
            RtcPeerConnection::set_onicecandidate,
            &self.webrtc,
        );

        init_weak_callback(
            &self,
            Self::on_negotiationneeded,
            &self.negotiationneeded_handler,
            RtcPeerConnection::set_onnegotiationneeded,
            &self.webrtc,
        );

        init_weak_callback(
            &self,
            Self::on_iceconnectionstatechange,
            &self.iceconnectionstatechange_handler,
            RtcPeerConnection::set_oniceconnectionstatechange,
            &self.webrtc,
        );

        init_weak_callback(
            &self,
            Self::on_icegatheringstatechange,
            &self.icegatheringstatechange_handler,
            RtcPeerConnection::set_onicegatheringstatechange,
            &self.webrtc,
        );

        init_weak_callback(
            &self,
            Self::on_signalingstatechange,
            &self.signalingstatechange_handler,
            RtcPeerConnection::set_onsignalingstatechange,
            &self.webrtc,
        );

        init_weak_callback(
            &self,
            Self::on_input,
            &self.input_handler,
            HtmlElement::set_oninput,
            &self.text,
        );

        self.send_offer().await;
    }

    async fn start_receiver(self: &Arc<Self>) {
        use crate::SendWebSocketMessage;
        use protocol::ClientMessage;

        self.websocket.send(ClientMessage::StartReceiver);
    }

    async fn send_offer(self: &Arc<Self>) {
        use crate::SendWebSocketMessage;
        use js_sys::Reflect;
        use protocol::ClientSenderMessage;
        use wasm_bindgen::{JsCast, JsValue};
        use wasm_bindgen_futures::JsFuture;
        use web_sys::RtcSessionDescriptionInit;

        let offer = JsFuture::from(self.webrtc.create_offer()).await.unwrap();

        let offer: &RtcSessionDescriptionInit = offer.as_ref().unchecked_ref();

        let _: JsValue = JsFuture::from(self.webrtc.set_local_description(offer))
            .await
            .unwrap();

        let sdp = Reflect::get(&offer, &JsValue::from_str("sdp"))
            .unwrap()
            .as_string()
            .unwrap();

        log::debug!("local offer: {:?}", sdp);
        self.websocket
            .send(ClientSenderMessage::Offer(SessionDescription(sdp)));
    }

    fn on_message(self: &Arc<Self>, ev: MessageEvent) {
        use crate::ParseWebSocketMessage;
        use protocol::ServerSenderMessage;
        use wasm_bindgen_futures::spawn_local;

        let message = ev.parse();
        match message {
            ServerSenderMessage::Answer(sdp) => {
                let self_arc = Arc::clone(self);
                spawn_local(async move { self_arc.on_answer(sdp).await });
            }
            ServerSenderMessage::IceCandidate(candidate) => {
                let self_arc = Arc::clone(self);
                spawn_local(async move { self_arc.on_remote_icecandidate(candidate).await });
            }
            ServerSenderMessage::AllIceCandidatesSent => {
                let self_arc = Arc::clone(self);
                spawn_local(async move { self_arc.on_remote_all_icecandidates_sent().await });
            }
        }
    }

    async fn on_answer(self: &Arc<Self>, answer: SessionDescription) {
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{RtcSdpType, RtcSessionDescriptionInit};

        log::debug!("remote answer: {:?}", answer);

        let mut remote_description = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
        let _: &mut _ = remote_description.sdp(&answer.0);

        let webrtc = self.webrtc.clone();
        let _: JsValue = JsFuture::from(webrtc.set_remote_description(&remote_description))
            .await
            .unwrap();
    }

    async fn on_remote_icecandidate(self: &Arc<Self>, ice_candidate: IceCandidate) {
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{RtcIceCandidate, RtcIceCandidateInit};

        log::debug!("remote ice candidate: {:?}", ice_candidate);

        let mut candidate = RtcIceCandidateInit::new(&ice_candidate.candidate);
        let _: &mut _ = candidate
            .sdp_mid(ice_candidate.sdp_mid.as_deref())
            .sdp_m_line_index(ice_candidate.sdp_mline_index);
        let candidate = RtcIceCandidate::new(&candidate).unwrap();

        let _: JsValue = JsFuture::from(
            self.webrtc
                .add_ice_candidate_with_opt_rtc_ice_candidate(Some(&candidate)),
        )
        .await
        .unwrap();
    }

    async fn on_remote_all_icecandidates_sent(self: &Arc<Self>) {
        log::debug!("remote all ice candidates sent");
    }

    fn on_icecandidate(self: &Arc<Self>, ev: RtcPeerConnectionIceEvent) {
        use crate::SendWebSocketMessage;
        use protocol::ClientSenderMessage;

        if let Some(candidate) = ev.candidate() {
            let candidate_str = candidate.candidate();
            match candidate_str.as_ref() {
                "" => {
                    log::debug!("local all ice candidates sent");
                    self.websocket
                        .send(ClientSenderMessage::AllIceCandidatesSent)
                }
                _ => {
                    let ice_candidate = IceCandidate {
                        candidate: candidate_str,
                        sdp_mid: candidate.sdp_mid(),
                        sdp_mline_index: candidate.sdp_m_line_index(),
                        username_fragment: None,
                    };
                    log::debug!("local ice candidate: {:?}", ice_candidate);
                    self.websocket
                        .send(ClientSenderMessage::IceCandidate(ice_candidate));
                }
            };
        }
    }

    fn on_negotiationneeded(self: &Arc<Self>, _: Event) {
        use wasm_bindgen_futures::spawn_local;

        let self_arc = Arc::clone(self);
        spawn_local(async move { self_arc.send_offer().await });
    }

    fn on_iceconnectionstatechange(self: &Arc<Self>, _: Event) {
        log::debug!(
            "ice connection state: {:?}",
            self.webrtc.ice_connection_state()
        );
    }

    fn on_icegatheringstatechange(self: &Arc<Self>, _: Event) {
        log::debug!(
            "ice gathering state: {:?}",
            self.webrtc.ice_gathering_state()
        );
    }

    fn on_signalingstatechange(self: &Arc<Self>, _: Event) {
        log::debug!("signaling state: {:?}", self.webrtc.signaling_state());
    }

    fn on_input(self: &Arc<Self>, ev: Event) {
        use wasm_bindgen::JsCast;

        let target: HtmlTextAreaElement = ev.target().unwrap().dyn_into().unwrap();
        // The null first byte is used because the operation fails if an empty string is sent
        let data = format!("\0{}", target.value());
        self.data_channel
            .send_with_u8_array(data.as_bytes())
            .unwrap();
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        use wasm_bindgen::JsCast;
        use web_sys::MediaStreamTrack;

        self.websocket.set_onmessage(None);
        self.webrtc.set_onicecandidate(None);
        self.webrtc.set_onnegotiationneeded(None);
        self.webrtc.set_oniceconnectionstatechange(None);
        self.webrtc.set_onicegatheringstatechange(None);
        self.webrtc.set_onsignalingstatechange(None);

        let _: Option<_> = self.websocket.close().ok();
        self.webrtc.close();
        self.data_channel.close();
        for track in self.media_stream.get_tracks().iter() {
            let track: Result<MediaStreamTrack, _> = track.dyn_into();
            if let Ok(track) = track {
                track.stop();
            }
        }

        self.video.remove();
        self.text.remove();
    }
}
