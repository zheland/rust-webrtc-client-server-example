#[derive(Debug)]
pub enum ChannelMessage {
    Data(Vec<u8>),
    Media(Vec<u8>),
}
