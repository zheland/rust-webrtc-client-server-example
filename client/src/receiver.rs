use core::cell::RefCell;

use async_std::sync::Arc;
use protocol::{IceCandidate, SessionDescription};
use wasm_bindgen::closure::Closure;
use web_sys::{
    Event, HtmlTextAreaElement, HtmlVideoElement, MediaStream, MessageEvent, RtcDataChannel,
    RtcDataChannelEvent, RtcPeerConnection, RtcPeerConnectionIceEvent, RtcTrackEvent, WebSocket,
};

use crate::ClosureCell1;

#[derive(Debug)]
pub struct Receiver {
    websocket: WebSocket,
    webrtc: RtcPeerConnection,
    videos: RefCell<Vec<HtmlVideoElement>>,
    text_areas: RefCell<Vec<HtmlTextAreaElement>>,
    data_channels: RefCell<Vec<RtcDataChannel>>,
    media_streams: RefCell<Vec<MediaStream>>,
    message_handler: ClosureCell1<MessageEvent>,
    icecandidate_handler: ClosureCell1<RtcPeerConnectionIceEvent>,
    negotiationneeded_handler: ClosureCell1<Event>,
    iceconnectionstatechange_handler: ClosureCell1<Event>,
    icegatheringstatechange_handler: ClosureCell1<Event>,
    signalingstatechange_handler: ClosureCell1<Event>,
    datachannel_handler: ClosureCell1<RtcDataChannelEvent>,
    track_handler: ClosureCell1<RtcTrackEvent>,
    datachannel_message_handlers: RefCell<Vec<Closure<dyn FnMut(MessageEvent)>>>,
}

impl Receiver {
    pub async fn new(addr: String) -> Arc<Self> {
        use crate::RtcConfigurationExt;
        use js_sys::Promise;
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{BinaryType, RtcConfiguration};

        let conf = RtcConfiguration::new().with_google_stun_server();
        let webrtc = RtcPeerConnection::new_with_configuration(&conf).unwrap();

        let websocket = WebSocket::new(addr.as_ref()).unwrap();
        websocket.set_binary_type(BinaryType::Arraybuffer);

        let web_socket_opened = Promise::new(&mut |resolve, reject| {
            websocket.set_onopen(Some(&resolve));
            websocket.set_onerror(Some(&reject));
        });
        let _: JsValue = JsFuture::from(web_socket_opened).await.unwrap();

        let receiver = Arc::new(Self {
            websocket,
            webrtc,
            videos: RefCell::new(Vec::new()),
            text_areas: RefCell::new(Vec::new()),
            data_channels: RefCell::new(Vec::new()),
            media_streams: RefCell::new(Vec::new()),
            message_handler: RefCell::new(None),
            icecandidate_handler: RefCell::new(None),
            negotiationneeded_handler: RefCell::new(None),
            iceconnectionstatechange_handler: RefCell::new(None),
            icegatheringstatechange_handler: RefCell::new(None),
            signalingstatechange_handler: RefCell::new(None),
            datachannel_handler: RefCell::new(None),
            track_handler: RefCell::new(None),
            datachannel_message_handlers: RefCell::new(Vec::new()),
        });

        receiver.init().await;

        receiver
    }

    async fn init(self: &Arc<Self>) {
        use crate::init_weak_callback;

        self.start_server_sender().await;

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
            Self::on_datachannel,
            &self.datachannel_handler,
            RtcPeerConnection::set_ondatachannel,
            &self.webrtc,
        );

        init_weak_callback(
            &self,
            Self::on_track,
            &self.track_handler,
            RtcPeerConnection::set_ontrack,
            &self.webrtc,
        );
    }

    async fn start_server_sender(self: &Arc<Self>) {
        use crate::SendWebSocketMessage;
        use protocol::ClientMessage;

        self.websocket.send(ClientMessage::StartSender);
    }

    fn on_message(self: &Arc<Self>, ev: MessageEvent) {
        use crate::ParseWebSocketMessage;
        use protocol::ServerSenderMessage;
        use wasm_bindgen_futures::spawn_local;

        let message = ev.parse();
        match message {
            ServerSenderMessage::Offer(sdp) => {
                let self_arc = Arc::clone(self);
                spawn_local(async move { self_arc.on_offer(sdp).await });
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

    async fn on_offer(self: &Arc<Self>, answer: SessionDescription) {
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{RtcSdpType, RtcSessionDescriptionInit};

        log::debug!("remote offer: {:?}", answer);

        let mut remote_description = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
        let _: &mut _ = remote_description.sdp(&answer.0);

        let webrtc = self.webrtc.clone();
        let _: JsValue = JsFuture::from(webrtc.set_remote_description(&remote_description))
            .await
            .unwrap();

        self.send_answer().await;
    }

    async fn send_answer(self: &Arc<Self>) {
        use crate::SendWebSocketMessage;
        use js_sys::Reflect;
        use protocol::ClientReceiverMessage;
        use wasm_bindgen::JsCast;
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::RtcSessionDescriptionInit;

        let answer = JsFuture::from(self.webrtc.create_answer()).await.unwrap();
        let answer: &RtcSessionDescriptionInit = answer.as_ref().unchecked_ref();

        let _: JsValue = JsFuture::from(self.webrtc.set_local_description(answer))
            .await
            .unwrap();

        let sdp = Reflect::get(&answer, &JsValue::from_str("sdp"))
            .unwrap()
            .as_string()
            .unwrap();

        log::debug!("local answer: {:?}", answer);
        self.websocket
            .send(ClientReceiverMessage::Answer(SessionDescription(sdp)));
    }

    async fn on_remote_icecandidate(self: &Arc<Self>, ice_candidate: IceCandidate) {
        crate::on_remote_icecandidate(&self.webrtc, ice_candidate).await;
    }

    async fn on_remote_all_icecandidates_sent(self: &Arc<Self>) {
        log::debug!("remote all ice candidates sent");
    }

    fn on_icecandidate(self: &Arc<Self>, ev: RtcPeerConnectionIceEvent) {
        use protocol::ClientSenderMessage;

        crate::on_icecandidate(&self.websocket, ev, ClientSenderMessage::IceCandidate);
    }

    fn on_negotiationneeded(self: &Arc<Self>, _: Event) {
        use wasm_bindgen_futures::spawn_local;

        let self_arc = Arc::clone(self);
        spawn_local(async move { self_arc.send_answer().await });
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

    fn on_datachannel(self: &Arc<Self>, ev: RtcDataChannelEvent) {
        use crate::{body, ElementExt};
        use js_sys::{ArrayBuffer, Uint8Array};
        use wasm_bindgen::JsCast;
        use web_sys::RtcDataChannelType;

        log::debug!("datachannel received");

        let text_area: HtmlTextAreaElement = body().add_child("textarea");
        text_area.set_read_only(true);

        let channel = ev.channel();
        channel.set_binary_type(RtcDataChannelType::Arraybuffer);

        let message_handler: Closure<dyn FnMut(MessageEvent)> = Closure::wrap(Box::new({
            let text_area = text_area.clone();
            move |ev| {
                let array_buffer: ArrayBuffer = ev.data().dyn_into().unwrap();
                let data = Uint8Array::new(&array_buffer).to_vec();
                let string = String::from_utf8_lossy(&data);

                text_area.set_value(string.as_ref());
            }
        }));
        channel.set_onmessage(Some(message_handler.as_ref().unchecked_ref()));

        self.text_areas.borrow_mut().push(text_area);
        self.data_channels.borrow_mut().push(channel);
        self.datachannel_message_handlers
            .borrow_mut()
            .push(message_handler);
    }

    fn on_track(self: &Arc<Self>, ev: RtcTrackEvent) {
        use crate::{body, ElementExt};
        use wasm_bindgen::JsCast;

        log::debug!("track received");
        web_sys::console::log_1(&wasm_bindgen::JsValue::from(&ev));

        let video: HtmlVideoElement = body().add_child("video");
        video.set_autoplay(true);
        let _: Option<_> = video.set_attribute("playsinline", "").ok();

        for media_stream in ev.streams().iter() {
            let media_stream: MediaStream = media_stream.dyn_into().unwrap();
            video.set_src_object(Some(&media_stream));
        }
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        use wasm_bindgen::JsCast;
        use web_sys::MediaStreamTrack;

        self.websocket.set_onmessage(None);
        self.webrtc.set_onicecandidate(None);
        self.webrtc.set_onnegotiationneeded(None);
        self.webrtc.set_oniceconnectionstatechange(None);
        self.webrtc.set_onicegatheringstatechange(None);
        self.webrtc.set_onsignalingstatechange(None);

        for data_channel in self.data_channels.borrow().iter() {
            data_channel.set_onmessage(None);
            data_channel.close();
        }

        for media_stream in self.media_streams.borrow().iter() {
            for track in media_stream.get_tracks().iter() {
                let track: Result<MediaStreamTrack, _> = track.dyn_into();
                if let Ok(track) = track {
                    track.stop();
                }
            }
        }

        let _: Option<_> = self.websocket.close().ok();
        self.webrtc.close();

        for video in self.videos.borrow().iter() {
            video.remove();
        }
        for text in self.text_areas.borrow().iter() {
            text.remove();
        }
    }
}
