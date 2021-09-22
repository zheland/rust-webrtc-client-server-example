use core::marker::PhantomData;

use futures::stream::SplitSink;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

#[derive(Debug)]
pub struct WebSocketSender<T> {
    sender: SplitSink<WebSocketStream<TcpStream>, Message>,
    _message: PhantomData<T>,
}

impl<T: Serialize> WebSocketSender<T> {
    pub fn new(sender: SplitSink<WebSocketStream<TcpStream>, Message>) -> Self {
        Self {
            sender,
            _message: PhantomData,
        }
    }

    pub async fn send(&mut self, message: T) {
        use bincode::serialize;
        use futures::SinkExt;

        let message: Vec<u8> = serialize(&message).unwrap();
        self.sender.send(Message::Binary(message)).await.unwrap();
    }
}
