mod synthesis;
mod commands;
mod config;
mod log;
mod event_handler;
mod type_map;
mod opt;

use config::Config;
use event_handler::Handler;
use type_map::{TextChannelId, ConfigData, ConnectedChannel};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tracing::{error, info};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::OffsetTime;
use songbird::SerenityInit;
use serenity::prelude::*;
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    let opt = opt::Opt::from_args();
    // サブコマンド
    if let Some(cmd) = opt.cmd {
        match cmd {
            opt::Command::Config(config_opt) => {
                opt::config(config_opt).unwrap();
            }
        }
        return;
    }

    let layer = tracing_subscriber::fmt::layer()
        .map_writer(|_| || log::LogWriter)
        .with_ansi(false)
        .with_timer(
            OffsetTime::new(time::macros::offset!(+9),
            time::macros::format_description!("[hour]:[minute]:[second]"))
        )
        .json()
        .pretty();
    tracing_subscriber::registry()
        .with(layer)
        .with(EnvFilter::from_default_env())
        .init();

    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(token, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .event_handler(Handler::default())
        .register_songbird()
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<TextChannelId>(Arc::new(Mutex::new(HashMap::default())));
        data.insert::<ConfigData>(Arc::new(Mutex::new(Config::load().unwrap_or_default())));
        data.insert::<ConnectedChannel>(Arc::new(Mutex::new(HashMap::new())));
    }

    tokio::spawn(async move {
        if let Err(why) = client.start().await {
            error!("Client error: {why:?}");
        }
    });

    let _ = tokio::signal::ctrl_c().await;
    info!("Received Ctrl-C, shutting down.");
}
