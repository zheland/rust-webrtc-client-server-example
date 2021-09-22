use core::fmt;
use std::sync::Arc;

use webrtc::media::rtp::rtp_receiver::RTCRtpReceiver;
use webrtc::media::track::track_remote::TrackRemote;

use crate::ChannelSender;

pub struct WebRtcMediaReceiver {
    channel_sender: ChannelSender,
    track: Arc<TrackRemote>,
}

impl WebRtcMediaReceiver {
    pub async fn new(
        channel_sender: ChannelSender,
        track: Arc<TrackRemote>,
        _: Option<Arc<RTCRtpReceiver>>,
    ) -> Arc<Self> {
        let receiver = Arc::new(Self {
            channel_sender,
            track,
        });

        receiver.init();

        receiver
    }

    fn init(self: &Arc<Self>) {
        self.spawn_thread()
    }

    fn spawn_thread(self: &Arc<Self>) {
        use tokio::spawn;
        use tokio::task::JoinHandle;

        let self_arc = Arc::clone(&self);
        let _: JoinHandle<()> = spawn(async move { self_arc.thread().await });
    }

    async fn thread(self: &Arc<Self>) {
        use crate::ChannelMessage;
        use webrtc_util::marshal::Marshal;
        use webrtc_util::marshal::MarshalSize;

        while let Ok((rtp, _)) = self.track.read_rtp().await {
            let len = rtp.marshal_size();
            let mut buf = vec![0; len];
            assert_eq!(rtp.marshal_to(&mut buf).unwrap(), len);
            self.channel_sender
                .send(ChannelMessage::Media(buf.to_vec()))
        }
    }
}

impl fmt::Debug for WebRtcMediaReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebRtcMediaReceiver")
            .finish_non_exhaustive()
    }
}
