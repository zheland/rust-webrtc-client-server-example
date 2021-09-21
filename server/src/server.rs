use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::{Channels, WebRtcApi};

#[derive(Debug)]
pub struct Server {
    webrtc_api: Arc<WebRtcApi>,
    channel: Arc<Mutex<Channels>>,
    listener: TcpListener,
}

impl Server {
    pub async fn new<Address: AsRef<str>>(addr: Address) -> Self {
        let webrtc_api = Arc::new(WebRtcApi::new());
        let channel = Arc::new(Mutex::new(Channels::new()));
        let listener = TcpListener::bind(addr.as_ref()).await.unwrap();

        log::info!("started on address: {}", addr.as_ref());

        Self {
            webrtc_api,
            channel,
            listener,
        }
    }

    pub async fn run(self) {
        use crate::Socket;
        use tokio::spawn;
        use tokio::task::JoinHandle;

        while let Ok((stream, addr)) = self.listener.accept().await {
            let channel = Arc::clone(&self.channel);
            let webrtc_api = Arc::clone(&self.webrtc_api);
            let _: JoinHandle<()> = spawn(async move {
                Socket::new(stream, addr, channel, webrtc_api)
                    .await
                    .run()
                    .await;
            });
        }
    }
}
