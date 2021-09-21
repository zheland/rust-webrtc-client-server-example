use core::cell::RefCell;

use async_std::sync::Arc;
use web_sys::Event;
use web_sys::{HtmlButtonElement, HtmlInputElement};

use crate::{ClosureCell1, Mode, Receiver, Sender};

#[derive(Debug)]
pub struct App {
    server_address_input: HtmlInputElement,
    start_sender_button: HtmlButtonElement,
    start_receiver_button: HtmlButtonElement,
    start_sender_click_handler: ClosureCell1<Event>,
    start_receiver_click_handler: ClosureCell1<Event>,
    mode: RefCell<Option<Mode>>,
}

impl App {
    pub fn new() -> Arc<Self> {
        use crate::default_server_address;
        use crate::{body, ElementExt};

        let server_address_input: HtmlInputElement = body().add_child("input");
        server_address_input.set_value(&default_server_address());
        let start_sender_button: HtmlButtonElement = body().add_child("button");
        start_sender_button.add_text("Start sender");
        let start_receiver_button: HtmlButtonElement = body().add_child("button");
        start_receiver_button.add_text("Start receiver");

        let app = Arc::new(App {
            server_address_input,
            start_sender_button,
            start_receiver_button,
            start_sender_click_handler: RefCell::new(None),
            start_receiver_click_handler: RefCell::new(None),
            mode: RefCell::new(None),
        });

        app.init();

        app
    }

    fn init(self: &Arc<Self>) {
        use crate::init_weak_callback;
        use web_sys::HtmlElement;

        init_weak_callback(
            &self,
            Self::on_start_sender_click,
            &self.start_sender_click_handler,
            HtmlElement::set_onclick,
            &self.start_sender_button,
        );

        init_weak_callback(
            &self,
            Self::on_start_receiver_click,
            &self.start_receiver_click_handler,
            HtmlElement::set_onclick,
            &self.start_receiver_button,
        );
    }

    fn set_start_buttons_inactive(&self) {
        self.server_address_input.set_read_only(true);
        self.start_sender_button.set_disabled(true);
        self.start_receiver_button.set_disabled(true);
    }

    fn on_start_sender_click(self: &Arc<Self>, _: Event) {
        use wasm_bindgen_futures::spawn_local;

        self.set_start_buttons_inactive();
        let addr = self.fix_and_get_server_address();
        let self_arc = Arc::clone(self);
        spawn_local(async move {
            let sender = Sender::new(addr).await;
            let prev = self_arc.mode.replace(Some(Mode::Sender(sender)));
            assert!(prev.is_none());
        });
    }

    fn on_start_receiver_click(self: &Arc<Self>, _: Event) {
        use wasm_bindgen_futures::spawn_local;

        self.set_start_buttons_inactive();
        let addr = self.fix_and_get_server_address();
        let self_arc = Arc::clone(self);
        spawn_local(async move {
            let receiver = Receiver::new(addr).await;
            let prev = self_arc.mode.replace(Some(Mode::Receiver(receiver)));
            assert!(prev.is_none());
        });
    }

    fn fix_and_get_server_address(&self) -> String {
        let addr = self.server_address_input.value();
        if addr.starts_with("ws://") || addr.starts_with("wss://") {
            addr
        } else {
            let addr = format!("ws://{}", &addr);
            self.server_address_input.set_value(&addr);
            addr
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.start_sender_button.set_onclick(None);
        self.start_receiver_button.set_onclick(None);
        self.server_address_input.remove();
        self.start_sender_button.remove();
        self.start_receiver_button.remove();
    }
}
