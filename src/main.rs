mod synthesis;
mod commands;
mod dictionary;
mod config;

use config::Config;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use chrono::Timelike;
use anyhow::Result;
use songbird::SerenityInit;
use serenity::{
    async_trait,
    prelude::*,
    model::{
        prelude::*,
        application::interaction::Interaction,
        gateway::Ready,
        id::{GuildId, ChannelId}
    }
};

const MAX_TEXT_LEN: usize = 255;

pub struct TextChannelId;

impl TypeMapKey for TextChannelId {
    type Value = Arc<Mutex<HashMap<GuildId, ChannelId>>>;
}

pub struct ConfigData;

impl TypeMapKey for ConfigData {
    type Value = Arc<Mutex<Config>>;
}

#[derive(Debug, Default)]
struct Handler {
    is_loop_running: AtomicBool
}

#[async_trait]
impl EventHandler for Handler {
    #[allow(clippy::single_match)]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                if let Err(why) = match command.data.name.as_str() {
                    "join" => commands::join::run(&ctx, &command).await,
                    "leave" => commands::leave::run(&ctx, &command).await,
                    "version" => commands::version::run(&ctx, &command).await,
                    "skip" => commands::skip::run(&ctx, &command).await,
                    "dictionary" => commands::dictionary::run(&ctx, &command).await,
                    "time-signal" => commands::time_signal::run(&ctx, &command).await,
                    "autojoin" => commands::autojoin::run(&ctx, &command).await,
                    _ => unimplemented!()
                } {
                    println!("Cannot respond to slash command: {why}");
                }
            },
            _ => {}
        }

        {
            let data_read = ctx.data.read().await;
            let config = data_read.get::<ConfigData>().unwrap();
            let _ = config.lock().unwrap().save();
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        for guild in &ready.guilds {
            let guild_id = guild.id;
            let _commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
                commands
                    .create_application_command(|cmd| commands::join::register(cmd))
                    .create_application_command(|cmd| commands::leave::register(cmd))
                    .create_application_command(|cmd| commands::version::register(cmd))
                    .create_application_command(|cmd| commands::skip::register(cmd))
                    .create_application_command(|cmd| commands::dictionary::register(cmd))
                    .create_application_command(|cmd| commands::time_signal::register(cmd))
                    .create_application_command(|cmd| commands::autojoin::register(cmd))
            }).await.unwrap();

            {
                let data_read = ctx.data.read().await;
                let config = data_read.get::<ConfigData>().unwrap();
                let _ = config.lock().unwrap().reload();
            }
        }
    }

    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx = Arc::new(ctx);
            tokio::spawn(async move {
                loop {
                    let time = std::time::SystemTime::
                        now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    if ((time % 3600) / 60, time % 60) == (0, 0) {
                        let local_hour = chrono::offset::Local::now().hour();
                        for guild in guilds.clone() {
                            let is_in_vc = guild.to_guild_cached(&ctx.cache)
                                .map(|guild| guild.voice_states.contains_key(&ctx.cache.current_user_id()));
                            if is_in_vc == Some(true) {
                                let ctx = Arc::clone(&ctx);
                                tokio::spawn(async move {
                                    let text = format!(
                                        "{}{}時をお知らせします。",
                                        if local_hour < 12 {"午前"} else {"午後"},
                                        local_hour % 12
                                    );
                                    let _ = speak(&ctx, guild, &text).await;
                                });
                            }
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            });

            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let self_id = ctx.cache.current_user_id();

        // 自身のメッセージは無視
        if msg.author.id == self_id {
            return;
        }

        let guild = msg.guild(&ctx.cache).unwrap();

        // 読み上げるテキストチャンネルを取得
        let Some(text_channel) = ({
            let data_read = ctx.data.read().await;
            let channel_id = data_read.get::<TextChannelId>().unwrap();
            let text_channel = channel_id.lock().unwrap().get(&guild.id).cloned();
            text_channel
        }) else { return; };

        // 自身がVCにいるときのみ読み上げる
        if msg.channel_id == text_channel && guild.voice_states.contains_key(&self_id) {
            let mut text = String::new();

            match msg.kind {
                MessageType::ThreadCreated => text.push_str("新規スレッド "),
                MessageType::InlineReply => text.push_str("リプライ "),
                _ => {}
            }
            if !msg.attachments.is_empty() {
                text.push_str("添付ファイル ");
            }

            let content = {
                let data_read = ctx.data.read().await;
                let config = data_read.get::<ConfigData>().unwrap();
                let mut config_lock = config.lock().unwrap();
                let dict = &config_lock.guild_config(guild.id).dictionary;
                dict.apply(&msg.content).unwrap_or(msg.content.clone())
            };

            text.push_str(&content);

            // 長文は省略
            if text.chars().count() > MAX_TEXT_LEN {
                text = format!("{} 以下省略", text.chars().take(MAX_TEXT_LEN).collect::<String>());
            }

            let _ = speak(&ctx, guild.id, text.trim()).await;
        }
    }

    /// 非botのユーザーが全員VCを抜けたら自動的に切断する
    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        // 自動入室
        let Some(guild_id) = new.guild_id else { return; };
        let autojoin = {
            let data_read = ctx.data.read().await;
            let config = data_read.get::<ConfigData>().unwrap();
            let mut config_lock = config.lock().unwrap();
            config_lock.guild_config(guild_id).autojoin
        };
        if autojoin && old.is_none() {
            if let Some(connect_to) = new.channel_id {
                let manager = songbird::get(&ctx).await.unwrap();
                let _ = manager.join(guild_id, connect_to).await;
                {
                    let data_read = ctx.data.read().await;
                    let channel_id = data_read.get::<TextChannelId>().unwrap();
                    let mut lock = channel_id.lock().unwrap();
                    lock.insert(guild_id, connect_to);
                }
            }
        }

        // 自動退室
        let Some(old) = old else { return; };

        let Some(channel_id) = old.channel_id else { return; };
        let Ok(Channel::Guild(channel)) = channel_id.to_channel(&ctx.http).await else { return; };

        let Ok(members) = channel.members(&ctx.cache).await else { return; };
        
        if members.is_empty() || members.iter().all(|member| member.user.bot) {
            let Some(guild_id) = old.guild_id else { return; };
            let manager = songbird::get(&ctx).await.unwrap();
            let _ = manager.leave(guild_id).await;
        }
    }
}

async fn speak(ctx: &Context, guild_id: GuildId, text: &str) -> Result<()> {
    let Some(manager) = songbird::get(ctx).await else {
        anyhow::bail!("Failed to retrieve Songbird voice client");
    };
    let Some(handle) = manager.get(guild_id) else {
        anyhow::bail!("Failed to retrieve Call handler");
    };
    let Ok(data) = synthesis::synthesis(text) else {
        anyhow::bail!("Failed to synthesis");
    };
    let input = synthesis::ffmpeg(&data);
    let mut handler = handle.lock().await;
    handler.enqueue_source(input);

    Ok(())
}

#[tokio::main]
async fn main() {
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
    }

    tokio::spawn(async move {
        if let Err(why) = client.start().await {
            println!("Client error: {why:?}");
        }
    });

    let _ = tokio::signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");
}
