mod synthesis;
mod commands;
mod dictionary;
mod config;

use config::Config;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use chrono::Timelike;
use tokio::sync::RwLock;
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
    type Value = Arc<RwLock<HashMap<GuildId, ChannelId>>>;
}

pub struct ConfigData;

impl TypeMapKey for ConfigData {
    type Value = Arc<RwLock<Config>>;
}

#[derive(Debug, Default)]
struct Handler {
    is_loop_running: AtomicBool
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                let options = &command.data.options;
                if let Err(why) = match command.data.name.as_str() {
                    "join" => commands::join::run(options, &ctx, &command).await,
                    "leave" => commands::leave::run(options, &ctx, &command).await,
                    "version" => commands::version::run(options, &ctx, &command).await,
                    "skip" => commands::skip::run(options, &ctx, &command).await,
                    "dictionary" => commands::dictionary::run(options, &ctx, &command).await,
                    _ => unimplemented!()
                } {
                    println!("Cannot respond to slash command: {why}");
                }
            },
            _ => {}
        }

        let data_read = {
            let data_read = ctx.data.read().await;
            data_read.get::<ConfigData>().unwrap().clone()
        };
        data_read.read().await.save();
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
            }).await.unwrap();

            let data_read = {
                let data_read = ctx.data.read().await;
                data_read.get::<ConfigData>().unwrap().clone()
            };
            let mut data_lock = data_read.write().await;
            data_lock.load(guild.id);
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
                                    synthesis::synthesis(&text).unwrap();
                                    duct::cmd!("ffmpeg", "-i", "temp.wav", "-ac", "2", "-ar", "48000", "sound.wav", "-y")
                                        .stdout_null()
                                        .stderr_null()
                                        .run()
                                        .unwrap();
    
                                    let manager = songbird::get(&ctx).await.unwrap();
    
                                    if let Some(handle) = manager.get(guild) {
                                        let mut handler = handle.lock().await;
                        
                                        let source = songbird::ffmpeg("sound.wav").await.unwrap();
                        
                                        let (track, _handle) = songbird::tracks::create_player(source);
                        
                                        handler.enqueue(track);
                                    } else {
                                        panic!();
                                    }
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
        let start = std::time::Instant::now();

        let self_id = ctx.cache.current_user_id();
        let guild = msg.guild(&ctx.cache).unwrap();

        // 自身のメッセージは無視
        if msg.author.id == self_id {
            return;
        }

        // 読み上げるテキストチャンネルを取得
        let Some(text_channel) = ({
            let data_read = {
                let data_read = ctx.data.read().await;
                data_read.get::<TextChannelId>().unwrap().clone()
            };
            let data_lock = data_read.read().await;
            data_lock.get(&guild.id).cloned()
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

            let data_read = {
                let data_read = ctx.data.read().await;
                data_read.get::<ConfigData>().unwrap().clone()
            };
            let content = data_read.read().await.0
                .get(&msg.guild_id.unwrap())
                .unwrap()
                .dictionary
                .apply(&msg.content);

            text.push_str(&content);

            // 長文は省略
            if text.chars().count() > MAX_TEXT_LEN {
                text = format!("{} 以下省略", text.chars().take(MAX_TEXT_LEN).collect::<String>());
            }

            synthesis::synthesis(&text).unwrap();

            duct::cmd!("ffmpeg", "-i", "temp.wav", "-ac", "2", "-ar", "48000", "sound.wav", "-y")
                .stdout_null()
                .stderr_null()
                .run()
                .unwrap();

            let manager = songbird::get(&ctx).await.unwrap();

            if let Some(handle) = manager.get(guild.id) {
                let mut handler = handle.lock().await;

                let source = songbird::ffmpeg("sound.wav").await.unwrap();

                let (track, _handle) = songbird::tracks::create_player(source);

                println!("{} ms", start.elapsed().as_millis());

                handler.enqueue(track);
            } else {
                panic!();
            }
        }
    }

    /// 非botのユーザーが全員VCを抜けたら自動的に切断する
    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, _new: VoiceState) {
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
        data.insert::<TextChannelId>(Arc::new(RwLock::new(HashMap::default())));
        data.insert::<ConfigData>(Arc::new(RwLock::new(Config::default())));
    }

    tokio::spawn(async move {
        if let Err(why) = client.start().await {
            println!("Client error: {why:?}");
        }
    });

    let _ = tokio::signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");
}
