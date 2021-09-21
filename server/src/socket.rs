use std::net::SocketAddr;
use std::sync::Arc;

use futures::stream::SplitSink;
use protocol::ClientMessage;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

use crate::{Channels, WebRtcApi, WebSocketReceiver};

#[derive(Debug)]
pub struct Socket {
    websocket_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
    websocket_receiver: WebSocketReceiver<ClientMessage>,
    addr: SocketAddr,
    channels: Arc<Mutex<Channels>>,
    webrtc_api: Arc<WebRtcApi>,
}

impl Socket {
    pub async fn new(
        stream: TcpStream,
        addr: SocketAddr,
        channels: Arc<Mutex<Channels>>,
        webrtc_api: Arc<WebRtcApi>,
    ) -> Self {
        use futures::StreamExt;
        use tokio_tungstenite::accept_async;

        let websocket = accept_async(stream).await.unwrap();
        let (websocket_sender, websocket_receiver) = websocket.split();
        let websocket_receiver = WebSocketReceiver::new(websocket_receiver);

        Self {
            channels,
            websocket_sender,
            websocket_receiver,
            addr,
            webrtc_api,
        }
    }

    pub async fn run(mut self) {
        use crate::{SocketReceiver, SocketSender};

        let addr = self.addr;
        log::info!("socket {}: opened", addr);

        match self.websocket_receiver.recv().await {
            Some(ClientMessage::StartReceiver) => {
                SocketReceiver::new(
                    self.websocket_sender,
                    self.websocket_receiver.into_stream(),
                    self.addr,
                    self.channels,
                    self.webrtc_api,
                )
                .await
                .run()
                .await
            }
            Some(ClientMessage::StartSender) => {
                SocketSender::new(
                    self.websocket_sender,
                    self.websocket_receiver.into_stream(),
                    self.addr,
                    self.channels,
                    self.webrtc_api,
                )
                .await
                .run()
                .await
            }
            None => {}
        }

        log::info!("socket {}: closed", addr);
    }
}
