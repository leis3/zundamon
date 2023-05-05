mod synthesis;
mod commands;
use songbird::SerenityInit;
use serenity::{
    async_trait,
    prelude::*,
    model::{
        prelude::*,
        application::interaction::{Interaction, InteractionResponseType},
        gateway::Ready,
        id::GuildId
    }
};

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
        let self_id = ctx.cache.current_user_id();
        let guild = msg.guild(&ctx.cache).unwrap();

        // 自身のメッセージは無視
        if msg.author.id == self_id {
            return;
        }

        // 自身がVCにいるときのみ読み上げる
        if guild.voice_states.contains_key(&self_id) {
            let join_handle = tokio::spawn(async move {
                synthesis::synthesis(&msg.content).unwrap()
            });

            let manager = songbird::get(&ctx).await.unwrap();

            if let Some(handle) = manager.get(guild.id) {
                let mut handler = handle.lock().await;

                let data = join_handle.await.unwrap();
                let cursor = std::io::Cursor::new(data);
                let reader = songbird::input::Reader::Extension(Box::new(cursor));
                let metadata = songbird::input::Metadata {
                    channels: Some(1),
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

                handler.play_source(source);
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

    tokio::spawn(async move {
        if let Err(why) = client.start().await {
            println!("Client error: {why:?}");
        }
    });

    let _ = tokio::signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");
}
