use protocol::IceCandidate;
use serde::Serialize;
use web_sys::{RtcConfiguration, RtcPeerConnection, RtcPeerConnectionIceEvent, WebSocket};

pub trait RtcConfigurationExt {
    fn with_google_stun_server(self) -> Self;
}

impl RtcConfigurationExt for RtcConfiguration {
    fn with_google_stun_server(mut self) -> Self {
        use js_sys::Array;
        use wasm_bindgen::JsValue;
        use web_sys::RtcIceServer;

        let ice_server_urls = vec![JsValue::from("stun:stun.l.google.com:19302")];
        let ice_server_urls: Array = ice_server_urls.into_iter().collect();
        let mut ice_server = RtcIceServer::new();
        let _: &mut _ = ice_server.urls(&JsValue::from(ice_server_urls));

        let ice_servers: Array = vec![ice_server].into_iter().collect();
        let _: &mut _ = self.ice_servers(&JsValue::from(ice_servers));

        self
    }
}

pub async fn on_remote_icecandidate(
    peer_connection: &RtcPeerConnection,
    ice_candidate: IceCandidate,
) {
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
        peer_connection.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&candidate)),
    )
    .await
    .unwrap();
}

pub fn on_icecandidate<T, F>(websocket: &WebSocket, ev: RtcPeerConnectionIceEvent, msg_fn: F)
where
    T: Serialize,
    F: FnOnce(IceCandidate) -> T,
{
    use crate::SendWebSocketMessage;
    use protocol::ClientSenderMessage;

    if let Some(candidate) = ev.candidate() {
        let candidate_str = candidate.candidate();
        match candidate_str.as_ref() {
            "" => {
                log::debug!("local all ice candidates sent");
                websocket.send(ClientSenderMessage::AllIceCandidatesSent)
            }
            _ => {
                let ice_candidate = IceCandidate {
                    candidate: candidate_str,
                    sdp_mid: candidate.sdp_mid(),
                    sdp_mline_index: candidate.sdp_m_line_index(),
                    username_fragment: None,
                };
                log::debug!("local ice candidate: {:?}", ice_candidate);
                websocket.send(msg_fn(ice_candidate));
            }
        };
    }
}
