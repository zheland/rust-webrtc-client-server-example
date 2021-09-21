use std::net::SocketAddr;
use std::sync::Arc;

use futures::stream::{SplitSink, SplitStream};
use protocol::ClientReceiverMessage;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

use crate::{Channels, WebRtcApi, WebRtcSender, WebSocketReceiver};

#[derive(Debug)]
pub struct SocketSender {
    addr: SocketAddr,
    websocket_receiver: WebSocketReceiver<ClientReceiverMessage>,
    webrtc_sender: Arc<WebRtcSender>,
}

impl SocketSender {
    pub async fn new(
        websocket_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
        websocket_receiver: SplitStream<WebSocketStream<TcpStream>>,
        addr: SocketAddr,
        channels: Arc<Mutex<Channels>>,
        webrtc_api: Arc<WebRtcApi>,
    ) -> Self {
        use crate::WebSocketSender;

        let channel_receiver = channels.lock().await.receiver();
        let websocket_sender = WebSocketSender::new(websocket_sender);
        let websocket_receiver = WebSocketReceiver::new(websocket_receiver);
        let webrtc_sender = WebRtcSender::new(webrtc_api, channel_receiver, websocket_sender).await;

        Self {
            addr,
            websocket_receiver,
            webrtc_sender,
        }
    }

    pub async fn run(mut self) {
        let addr = self.addr;
        log::info!("sender socket {}: opened", addr);

        while let Some(message) = self.websocket_receiver.recv().await {
            log::debug!("sender socket {}: message: {:?}", addr, message);
            match message {
                ClientReceiverMessage::Answer(offer) => {
                    self.webrtc_sender.on_answer(offer).await;
                }
                ClientReceiverMessage::IceCandidate(candidate) => {
                    self.webrtc_sender.on_remote_icecandidate(candidate).await;
                }
                ClientReceiverMessage::AllIceCandidatesSent => {
                    self.webrtc_sender.on_all_remote_icecandidates_sent().await;
                }
            }
        }

        log::info!("sender socket {}: closed", addr);
    }
}
