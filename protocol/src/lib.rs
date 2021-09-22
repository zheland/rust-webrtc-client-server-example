#![warn(
    clippy::all,
    rust_2018_idioms,
    missing_copy_implementations,
    missing_debug_implementations,
    single_use_lifetimes,
    trivial_casts,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SessionDescription(pub String);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
    pub username_fragment: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ClientMessage {
    StartReceiver,
    StartSender,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ClientSenderMessage {
    Offer(SessionDescription),
    IceCandidate(IceCandidate),
    AllIceCandidatesSent,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ClientReceiverMessage {
    Answer(SessionDescription),
    IceCandidate(IceCandidate),
    AllIceCandidatesSent,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ServerReceiverMessage {
    Answer(SessionDescription),
    IceCandidate(IceCandidate),
    AllIceCandidatesSent,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ServerSenderMessage {
    Offer(SessionDescription),
    IceCandidate(IceCandidate),
    AllIceCandidatesSent,
}
