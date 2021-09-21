use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Options {
    /// IP address to bind
    #[clap(short, long, default_value = "0.0.0.0")]
    address: String,
    /// Port number
    #[clap(short, long, default_value = "9010")]
    port: String,
}

pub async fn app() {
    use crate::Server;

    env_logger::init();
    let opts: Options = Options::parse();
    let addr = format!("{}:{}", opts.address, opts.port);
    let server = Server::new(addr).await;
    Server::run(server).await;
}
