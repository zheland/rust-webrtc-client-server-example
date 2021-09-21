use async_std::sync::Arc;

use crate::{Receiver, Sender};

#[derive(Debug)]
pub enum Mode {
    Sender(Arc<Sender>),
    Receiver(Arc<Receiver>),
}
