mod synthesis;
mod commands;
mod dictionary;

use dictionary::Dictionary;
use std::sync::Arc;
use std::collections::HashMap;
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

pub struct TextChannelId;

impl TypeMapKey for TextChannelId {
    type Value = Arc<RwLock<HashMap<GuildId, ChannelId>>>;
}

pub struct DictData;

impl TypeMapKey for DictData {
    type Value = Arc<RwLock<Dictionary>>;
}

#[derive(Debug, Default)]
struct Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
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
            }).await.unwrap();
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
        let text_channel = {
            let data_read = {
                let data_read = ctx.data.read().await;
                data_read.get::<TextChannelId>().unwrap().clone()
            };
            let data_lock = data_read.read().await;
            data_lock.get(&guild.id).cloned()
        }.unwrap();

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

            text.push_str(&msg.content);

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
        data.insert::<DictData>(Arc::new(RwLock::new(Dictionary::new())));
    }

    tokio::spawn(async move {
        if let Err(why) = client.start().await {
            println!("Client error: {why:?}");
        }
    });

    let _ = tokio::signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");
}
