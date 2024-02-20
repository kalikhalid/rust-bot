use anyhow::{Context, Result};
use log::*;
use std::sync::Arc;


mod bot;
mod config;
mod controller;
mod api;

#[derive(Debug)]
struct Code {
    id: i32,
    code: String,
    activations: u8,
}

const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    std::env::set_var("RUST_LOG", "info");
    let config = Arc::new(config::read_config());
    info!("starting with config: {config:#?}");
    let mut db = controller::Controller::open(&config)?;
    db.migrate()?;
    drop(db);

    let bot = bot::MyBot::new(config.clone()).await?;
    let (bot_handle, _) = bot.spawn();
    bot_handle.await?;

    Ok(())
}