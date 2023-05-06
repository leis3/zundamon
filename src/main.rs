mod synthesis;
mod commands;

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use songbird::SerenityInit;
use serenity::{
    async_trait,
    prelude::*,
    model::{
        prelude::*,
        application::interaction::{Interaction, InteractionResponseType},
        gateway::Ready,
        id::{GuildId, ChannelId}
    }
};

struct TextChannelId;

impl TypeMapKey for TextChannelId {
    type Value = Arc<RwLock<HashMap<GuildId, ChannelId>>>;
}

#[derive(Debug, Default)]
struct Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let options = &command.data.options;
            let content = match command.data.name.as_str() {
                "join" => commands::join::run(options, &ctx, &command).await,
                "leave" => commands::leave::run(options, &ctx, &command).await,
                _ => "not implemented".to_string()
            };

            if let Err(why) = command.create_interaction_response(&ctx.http, |response| {
                response.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(content))
            }).await {
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
            let join_handle = tokio::spawn(async move {
                synthesis::synthesis(&msg.content).unwrap()
            });

            let manager = songbird::get(&ctx).await.unwrap();

            if let Some(handle) = manager.get(guild.id) {
                let mut handler = handle.lock().await;

                // ずんだもんが2倍速で喋る問題を修正するため、2チャンネルに変換する
                let data = {
                    let data = join_handle.await.unwrap();
                    let reader = hound::WavReader::new(data.as_slice()).unwrap();
                    let spec = hound::WavSpec {
                        channels: 2,
                        ..reader.spec()
                    };
                    let mut buf = std::io::Cursor::new(Vec::with_capacity(data.len() * 2));
                    let mut writer = hound::WavWriter::new(&mut buf, spec).unwrap();
                    for sample in reader.into_samples::<i16>() {
                        let sample = sample.unwrap();
                        writer.write_sample(sample).unwrap();
                        writer.write_sample(sample).unwrap();
                    }
                    writer.finalize().unwrap();
                    buf
                };
                let reader = songbird::input::Reader::from_memory(data.into_inner());
                let metadata = songbird::input::Metadata {
                    channels: Some(2),
                    sample_rate: Some(24000),
                    ..Default::default()
                };
                let source = songbird::input::Input::new(
                    false,
                    reader,
                    songbird::input::codec::Codec::Pcm,
                    songbird::input::Container::Raw,
                    Some(metadata)
                );
                let (track, handle) = songbird::tracks::create_player(source);

                println!("{} ms", start.elapsed().as_millis());

                handler.enqueue(track);
            } else {
                assert!(false);
            }
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
    }

    tokio::spawn(async move {
        if let Err(why) = client.start().await {
            println!("Client error: {why:?}");
        }
    });

    let _ = tokio::signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");
}
