use web_sys::RtcConfiguration;

pub trait RtcConfigurationExt {
    fn with_google_stun_server(self) -> Self;
}

impl RtcConfigurationExt for RtcConfiguration {
    fn with_google_stun_server(mut self) -> Self {
        use js_sys::Array;
        use wasm_bindgen::JsValue;
        use web_sys::RtcIceServer;

        let ice_server_urls = vec![JsValue::from("stun:stun.l.google.com:19302")];
        let ice_server_urls: Array = ice_server_urls.into_iter().collect();
        let mut ice_server = RtcIceServer::new();
        let _: &mut _ = ice_server.urls(&JsValue::from(ice_server_urls));

        let ice_servers: Array = vec![ice_server].into_iter().collect();
        let _: &mut _ = self.ice_servers(&JsValue::from(ice_servers));

        self
    }
}
