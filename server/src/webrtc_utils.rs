use protocol::IceCandidate;
use serde::Serialize;
use tokio::sync::Mutex;
use webrtc::peer::ice::ice_candidate::RTCIceCandidate;
use webrtc::peer::peer_connection::RTCPeerConnection;

use crate::websocket_sender::WebSocketSender;

pub async fn add_remote_icecandidate(
    peer_connection: &RTCPeerConnection,
    candidate: IceCandidate,
    delayed: &Mutex<Vec<IceCandidate>>,
) {
    use webrtc::peer::ice::ice_candidate::RTCIceCandidateInit;

    if peer_connection.remote_description().await.is_some() {
        let candidate = RTCIceCandidateInit {
            candidate: candidate.candidate,
            sdp_mid: candidate.sdp_mid.unwrap_or_else(|| String::new()),
            sdp_mline_index: candidate.sdp_mline_index.unwrap_or(0),
            username_fragment: candidate.username_fragment.unwrap_or_else(|| String::new()),
        };

        peer_connection.add_ice_candidate(candidate).await.unwrap();
    } else {
        delayed.lock().await.push(candidate);
    }
}

pub async fn send_local_icecandidate<T, F>(
    websocket_sender: &Mutex<WebSocketSender<T>>,
    ice_candidate: Option<RTCIceCandidate>,
    candidate_msg_fn: F,
    all_candidates_sent_msg_fn: T,
) where
    T: Serialize,
    F: FnOnce(IceCandidate) -> T,
{
    if let Some(ice_candidate) = ice_candidate {
        let json = ice_candidate.to_json().await.unwrap();

        websocket_sender
            .lock()
            .await
            .send(candidate_msg_fn(IceCandidate {
                candidate: json.candidate,
                sdp_mid: Some(json.sdp_mid),
                sdp_mline_index: Some(json.sdp_mline_index),
                username_fragment: Some(json.username_fragment),
            }))
            .await
    } else {
        websocket_sender
            .lock()
            .await
            .send(all_candidates_sent_msg_fn)
            .await
    }
}
