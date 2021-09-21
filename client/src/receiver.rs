use async_std::sync::Arc;
use web_sys::HtmlDivElement;

#[derive(Debug)]
pub struct Receiver {
    placeholder: HtmlDivElement,
}

impl Receiver {
    pub async fn new(_addr: String) -> Arc<Self> {
        use crate::{body, ElementExt};

        let placeholder: HtmlDivElement = body().add_child("div");
        placeholder.add_text("Not yet implemented");

        Arc::new(Self { placeholder })
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.placeholder.remove();
    }
}
