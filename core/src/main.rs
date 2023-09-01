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
use tokio::io::AsyncBufReadExt;
use tracing::{error, info};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::OffsetTime;
use serenity::prelude::*;
use structopt::StructOpt;

#[allow(clippy::single_match)]
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

    let voice = songbird::Songbird::serenity();
    let builder = Client::builder(token, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT);
    let mut client = songbird::serenity::register_with(builder, voice.clone())
        .event_handler(Handler::default())
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<TextChannelId>(Arc::new(Mutex::new(HashMap::default())));
        data.insert::<ConfigData>(Arc::new(Mutex::new(Config::load().unwrap_or_default())));
        data.insert::<ConnectedChannel>(Arc::new(Mutex::new(HashMap::new())));
    }

    let (tx_stdout, mut rx_stdout) = tokio::sync::mpsc::unbounded_channel::<String>();
    let (tx_exit, rx_exit) = tokio::sync::oneshot::channel();

    // コンソールから標準入力を受け取ってチャネルに送るスレッド
    tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut lines = tokio::io::BufReader::new(stdin).lines();
        while let Some(line) = lines.next_line().await.unwrap() {
            let _ = tx_stdout.send(line);
        }
    });

    // 標準入力をチャネルから受け取ってexitを処理する
    tokio::spawn(async move {
        while let Some(line) = rx_stdout.recv().await {
            match line.as_str() {
                "stop" => {
                    tx_exit.send(()).unwrap();
                    break;
                },
                _ => {}
            }
        }
    });

    tokio::select! {
        result = client.start() => {
            if let Err(why) = result {
                error!("Client error: {why:?}");
            }
        }
        _ = rx_exit => {
            for &guild in &client.cache_and_http.cache.guilds() {
                let _ = voice.leave(guild).await;
            }
            client.shard_manager.lock().await.shutdown_all().await;
            println!("Press Enter");
        }
    }

    info!("shutting down.");
}
