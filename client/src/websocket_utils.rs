use serde::{Deserialize, Serialize};
use web_sys::{MessageEvent, WebSocket};

pub trait ParseWebSocketMessage<T> {
    fn parse(&self) -> T;
}

pub trait SendWebSocketMessage<T> {
    fn send(&self, message: T);
}

#[allow(single_use_lifetimes)] // false positive
impl<T> ParseWebSocketMessage<T> for MessageEvent
where
    T: for<'a> Deserialize<'a>,
{
    fn parse(&self) -> T {
        use bincode::deserialize;
        use js_sys::{ArrayBuffer, Uint8Array};
        use wasm_bindgen::JsCast;

        let array_buffer: ArrayBuffer = self.data().dyn_into().unwrap();
        let data = Uint8Array::new(&array_buffer).to_vec();
        deserialize(&data).unwrap()
    }
}

impl<T> SendWebSocketMessage<T> for WebSocket
where
    T: Serialize,
{
    fn send(&self, message: T) {
        use bincode::serialize;

        let request: Vec<u8> = serialize(&message).unwrap();
        self.send_with_u8_array(&request).unwrap();
    }
}
