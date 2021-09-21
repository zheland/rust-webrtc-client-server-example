use core::marker::PhantomData;

use futures::stream::SplitStream;
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

#[derive(Debug)]
pub struct WebSocketReceiver<T> {
    receiver: SplitStream<WebSocketStream<TcpStream>>,
    _message: PhantomData<T>,
}

#[allow(single_use_lifetimes)] // false positive
impl<T: for<'a> Deserialize<'a>> WebSocketReceiver<T> {
    pub fn new(receiver: SplitStream<WebSocketStream<TcpStream>>) -> Self {
        Self {
            receiver,
            _message: PhantomData,
        }
    }

    pub async fn recv(&mut self) -> Option<T> {
        use bincode::deserialize;
        use futures::StreamExt;

        let message = self.receiver.next().await.unwrap().unwrap();
        match message {
            Message::Binary(data) => Some(deserialize(&data[..]).unwrap()),
            Message::Close(_) => None,
            _ => {
                log::info!("invalid message: {:?}", message);
                None
            }
        }
    }

    pub fn into_stream(self) -> SplitStream<WebSocketStream<TcpStream>> {
        self.receiver
    }
}
