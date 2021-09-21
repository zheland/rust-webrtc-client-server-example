use std::net::SocketAddr;
use std::sync::Arc;

use futures::stream::{SplitSink, SplitStream};
use protocol::ClientSenderMessage;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

use crate::{Channels, WebRtcApi, WebRtcReceiver, WebSocketReceiver};

#[derive(Debug)]
pub struct SocketReceiver {
    addr: SocketAddr,
    websocket_receiver: WebSocketReceiver<ClientSenderMessage>,
    webrtc_receiver: Arc<WebRtcReceiver>,
}

impl SocketReceiver {
    pub async fn new(
        websocket_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
        websocket_receiver: SplitStream<WebSocketStream<TcpStream>>,
        addr: SocketAddr,
        channels: Arc<Mutex<Channels>>,
        webrtc_api: Arc<WebRtcApi>,
    ) -> Self {
        use crate::WebSocketSender;

        let channel_sender = channels.lock().await.sender();
        let websocket_sender = WebSocketSender::new(websocket_sender);
        let websocket_receiver = WebSocketReceiver::new(websocket_receiver);
        let webrtc_receiver =
            WebRtcReceiver::new(webrtc_api, channel_sender, websocket_sender).await;

        Self {
            addr,
            websocket_receiver,
            webrtc_receiver,
        }
    }

    pub async fn run(mut self) {
        let addr = self.addr;
        log::info!("receiver socket {}: opened", addr);

        while let Some(message) = self.websocket_receiver.recv().await {
            log::debug!("receiver socket {}: message: {:?}", addr, message);
            match message {
                ClientSenderMessage::Offer(offer) => {
                    self.webrtc_receiver.on_offer(offer).await;
                }
                ClientSenderMessage::IceCandidate(candidate) => {
                    self.webrtc_receiver.on_remote_icecandidate(candidate).await;
                }
                ClientSenderMessage::AllIceCandidatesSent => {
                    self.webrtc_receiver
                        .on_all_remote_icecandidates_sent()
                        .await;
                }
            }
        }

        log::info!("receiver socket {}: closed", addr);
    }
}
