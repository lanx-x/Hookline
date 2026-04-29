mod channels;
mod config;
mod handler;
mod notification;
mod server;

use std::path::PathBuf;

fn main() {
    env_logger::init();

    let config_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config.yaml"));

    let config: config::Config = config::Config::load(&config_path).unwrap_or_else(|e| {
        eprintln!("config error: {e}");
        std::process::exit(1);
    });

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");

    let channels: Vec<Box<dyn channels::Channel>> =
        runtime.block_on(channels::build_channels(&config.channels));

    runtime.block_on(server::run(config.server, config.endpoints, channels));
}
