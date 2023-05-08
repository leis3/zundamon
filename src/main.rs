mod synthesis;
mod commands;

use std::sync::Arc;
use std::collections::HashMap;
use rubato::Resampler;
use tokio::sync::RwLock;
use songbird::{
    SerenityInit,
    input::{
        Codec,
        Reader,
        Input,
        Metadata,
        Container
    }
};
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

                synthesis::synthesis(&text).unwrap()
            });

            let manager = songbird::get(&ctx).await.unwrap();

            if let Some(handle) = manager.get(guild.id) {
                let mut handler = handle.lock().await;

                let source = source(join_handle.await.unwrap());

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

fn source(data: Vec<u8>) -> Input {
    let rdr = hound::WavReader::new(data.as_slice()).unwrap();
    let spec = rdr.spec();
    let samples = rdr.into_samples::<i16>().map(|s| s.unwrap() as f32).collect::<Vec<_>>();

    let params = rubato::InterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        oversampling_factor: 256,
        interpolation: rubato::InterpolationType::Linear,
        window: rubato::WindowFunction::BlackmanHarris2
    };
    let mut resampler = rubato::SincFixedIn::<f32>::new(
        48000. / spec.sample_rate as f64,
        2.0,
        params,
        samples.len(),
        spec.channels as usize
    ).unwrap();

    let resampled = resampler.process(&[samples], None).unwrap().pop().unwrap();

    let mut buf = std::io::Cursor::new(Vec::with_capacity(data.len() * 2));
    
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48000,
        ..spec
    };

    let mut wtr = hound::WavWriter::new(&mut buf, spec).unwrap();
    for sample in resampled {
        wtr.write_sample(sample as i16).unwrap();
        wtr.write_sample(sample as i16).unwrap();
    }
    wtr.finalize().unwrap();

    let metadata = Metadata {
        channels: Some(2),
        sample_rate: Some(48000),
        ..Default::default()
    };

    let mut file = std::fs::File::create("wav/voice.wav").unwrap();
    use std::io::Write;
    file.write_all(&mut buf.get_ref().clone()).unwrap();

    let rdr = Reader::from_memory(buf.into_inner());

    Input::new(
        true,
        rdr,
        Codec::Pcm,
        Container::Raw,
        Some(metadata)
    )
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
