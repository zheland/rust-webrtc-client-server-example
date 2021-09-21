#![warn(
    clippy::all,
    rust_2018_idioms,
    missing_copy_implementations,
    missing_debug_implementations,
    single_use_lifetimes,
    trivial_casts,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

mod app;
mod html;
mod mode;
mod params;
mod receiver;
mod sender;
mod weak_callback;
mod webrtc_configuration;
mod websocket;

use app::App;
use html::{body, navigator, ElementExt};
use mode::Mode;
use params::default_server_address;
use receiver::Receiver;
use sender::Sender;
use weak_callback::{init_weak_callback, ClosureCell1};
use webrtc_configuration::RtcConfigurationExt;
use websocket::{ParseWebSocketMessage, SendWebSocketMessage};

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    let _: &mut _ = Box::leak(Box::new(App::new()));
}
