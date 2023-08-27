use crate::commands;
use crate::synthesis;
use crate::type_map::{TextChannelId, ConfigData, ConnectedChannel};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::{Timelike, Datelike};
use anyhow::Result;
use tracing::{error, info};
use serenity::{
    async_trait,
    prelude::*,
    model::{
        prelude::*,
        application::interaction::Interaction,
        gateway::Ready,
        id::GuildId
    }
};

const MAX_TEXT_LEN: usize = 255;

#[derive(Debug, Default)]
pub struct Handler {
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
                    "status" => commands::status::run(&ctx, &command).await,
                    "speaker" => commands::speaker::run(&ctx, &command).await,
                    "log" => commands::log::run(&ctx, &command).await,
                    _ => unimplemented!()
                } {
                    error!("Cannot respond to slash command: {why}");
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
        synthesis::initialize();
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
                    .create_application_command(|cmd| commands::status::register(cmd))
                    .create_application_command(|cmd| commands::speaker::register(cmd))
                    .create_application_command(|cmd| commands::log::register(cmd))
            }).await.unwrap();

            {
                let data_read = ctx.data.read().await;
                let config = data_read.get::<ConfigData>().unwrap();
                let _ = config.lock().unwrap().reload();
            }
        }
        ctx.set_activity(Activity::playing(format!("v{}", env!("CARGO_PKG_VERSION")))).await;
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
                        for guild in guilds.clone() {
                            let is_in_vc = guild.to_guild_cached(&ctx.cache)
                                .map(|guild| guild.voice_states.contains_key(&ctx.cache.current_user_id()));
                            if is_in_vc == Some(true) {
                                let ctx = Arc::clone(&ctx);
                                tokio::spawn(async move {
                                    let text = time_message();
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
                dict.apply(&msg.content)
                    .unwrap_or(msg.content.clone())
                    .replace("\n", "、")
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
        // 自動退室

        let Some(guild_id) = new.guild_id else { return; };
        let Some(voice_channel) = ({
            let data_read = ctx.data.read().await;
            let connected = data_read.get::<ConnectedChannel>().unwrap();
            let lock = connected.lock().unwrap();
            lock.get(&guild_id).cloned()
        }) else { return; };

        // VCから退出あるいは別のVCに移動
        if old.and_then(|state| state.channel_id) == Some(voice_channel) &&
            !new.channel_id.map_or(false, |id| id == voice_channel)
        {
            let Ok(Channel::Guild(channel)) = voice_channel.to_channel(&ctx.http).await else { return; };
            let Ok(members) = channel.members(&ctx.cache).await else { return; };

            if members.is_empty() || members.iter().all(|member| member.user.bot) {
                info!("auto disconnect");
                let manager = songbird::get(&ctx).await.unwrap();

                {
                    let data_read = ctx.data.read().await;
                    let connected = data_read.get::<ConnectedChannel>().unwrap();
                    let mut lock = connected.lock().unwrap();
                    lock.remove(&guild_id);
                }

                if let Some(handle) = manager.get(guild_id) {
                    let handler = handle.lock().await;
                    handler.queue().modify_queue(|q| q.clear());
                }
                let _ = manager.leave(guild_id).await;

            }
        }
    }

    async fn resume(&self, ctx: Context, resumed_event: ResumedEvent) {
        info!(resumed_event = ?resumed_event, "resume event");
        let connected = {
            let data_read = ctx.data.read().await;
            let connected = data_read.get::<ConnectedChannel>().unwrap();
            let lock = connected.lock().unwrap();
            lock.iter().map(|(&g, &c)| (g, c)).collect::<Vec<_>>()
        };
        let manager = songbird::get(&ctx).await.unwrap();
        for (guild_id, channel_id) in connected {
            let _ = manager.join(guild_id, channel_id).await;
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
    let speaker_id = {
        let data_read = ctx.data.read().await;
        let config = data_read.get::<ConfigData>().unwrap();
        let mut config_lock = config.lock().unwrap();
        config_lock.guild_config(guild_id).speaker_id
    };
    let Ok(data) = synthesis::synthesis(text, speaker_id) else {
        anyhow::bail!("Failed to synthesis");
    };
    let input = synthesis::to_input(&data);
    let mut handler = handle.lock().await;
    handler.enqueue_source(input);

    Ok(())
}

fn time_message() -> String {
    let now = chrono::Utc::now().with_timezone(&chrono_tz::Japan);
    let local_hour = now.hour();
    let weekday_str = ["月", "火", "水", "木", "金", "土", "日"];
    format!(
        "{}{}{}時をお知らせします。",
        if local_hour == 0 {
            format!("{}月{}日{}曜日 ",
                now.month(),
                now.day(),
                weekday_str[now.weekday().number_from_monday() as usize - 1]
            )
        } else {"".into()},
        if local_hour < 12 {"午前"} else {"午後"},
        local_hour % 12
    )
}
