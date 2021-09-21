use core::fmt;

use interceptor::registry::Registry;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::{APIBuilder, API};
use webrtc::peer::configuration::RTCConfiguration;
use webrtc::peer::ice::ice_server::RTCIceServer;
use webrtc::peer::peer_connection::RTCPeerConnection;

pub struct WebRtcApi {
    api: API,
}

impl WebRtcApi {
    pub fn new() -> Self {
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs().unwrap();

        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine).unwrap();

        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        Self { api }
    }

    pub fn default_config() -> RTCConfiguration {
        RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    pub async fn new_peer_connection(&self) -> RTCPeerConnection {
        self.api
            .new_peer_connection(Self::default_config())
            .await
            .unwrap()
    }
}

impl fmt::Debug for WebRtcApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebRtc").finish_non_exhaustive()
    }
}
