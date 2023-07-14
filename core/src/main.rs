mod synthesis;
mod commands;
mod config;
mod log;
mod event_handler;
mod type_map;

use config::Config;
use event_handler::Handler;
use type_map::{TextChannelId, ConfigData, ConnectedChannel};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use songbird::SerenityInit;
use structopt::StructOpt;
use tracing_subscriber::Layer;
use tracing_subscriber::prelude::*;
use serenity::{
    prelude::*,
    model::prelude::*
};



#[derive(Debug, StructOpt)]
struct Opt {
    /// Discord Webhoook URL to send logs to
    #[structopt(short, long)]
    pub log_webhook: Option<String>
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    if let Some(url) = &opt.log_webhook {
        log::LOG_WEBHOOK.get_or_init(|| url.clone());
        let layer = tracing_subscriber::fmt::layer()
            .map_writer(|_| || log::LogWriter)
            .without_time()
            .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                metadata.target().contains("core") && metadata.level() <= &tracing::Level::DEBUG
            }));
        tracing_subscriber::registry()
            .with(layer)
            .init();
    } else {
        let layer = tracing_subscriber::fmt::layer()
            .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                metadata.target().contains("core") && metadata.level() <= &tracing::Level::DEBUG
            }));
        tracing_subscriber::registry()
            .with(layer)
            .init();
    }

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
