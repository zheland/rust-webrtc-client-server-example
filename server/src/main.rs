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

mod app;
mod channel;
mod channel_message;
mod channel_receiver;
mod channel_sender;
mod channels;
mod server;
mod socket;
mod socket_receiver;
mod socket_sender;
mod weak_callback;
mod webrtc_api;
mod webrtc_data_receiver;
mod webrtc_media_receiver;
mod webrtc_receiver;
mod webrtc_sender;
mod webrtc_utils;
mod websocket_receiver;
mod websocket_sender;

use app::app;
use channel::{Channel, ChannelId};
use channel_message::ChannelMessage;
use channel_receiver::ChannelReceiver;
use channel_sender::ChannelSender;
use channels::Channels;
use server::Server;
use socket::Socket;
use socket_receiver::SocketReceiver;
use socket_sender::SocketSender;
use weak_callback::WeakAsyncCallback;
use webrtc_api::WebRtcApi;
use webrtc_data_receiver::WebRtcDataReceiver;
use webrtc_media_receiver::WebRtcMediaReceiver;
use webrtc_receiver::WebRtcReceiver;
use webrtc_sender::WebRtcSender;
use webrtc_utils::{add_remote_icecandidate, send_local_icecandidate};
use websocket_receiver::WebSocketReceiver;
use websocket_sender::WebSocketSender;

#[tokio::main]
pub async fn main() {
    app().await
}
