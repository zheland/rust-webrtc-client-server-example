#[derive(Debug)]
pub enum ChannelMessage {
    Data(Vec<u8>),
    Video(Vec<u8>),
    Audio(Vec<u8>),
}
