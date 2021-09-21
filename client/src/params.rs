pub fn default_server_address() -> String {
    const FALLBACK_ADDRESS: &str = "ws://localhost:9010";

    use js_sys::{JsString, Reflect};
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::window;

    window()
        .and_then(|window| Reflect::get(&window, &JsValue::from_str("server_address")).ok())
        .and_then(|addr| addr.dyn_into().ok())
        .map(|addr: JsString| addr.into())
        .unwrap_or(FALLBACK_ADDRESS.to_owned())
}
