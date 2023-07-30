use crate::{TextChannelId, ConnectedChannel};
use tracing::{debug, error};
use serenity::prelude::*;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::id::GuildId;
use serenity::model::channel::ChannelType;
use serenity::model::application::{
    command::CommandOptionType,
    interaction::{
        InteractionResponseType,
        application_command::{
            CommandDataOptionValue,
            ApplicationCommandInteraction
        }
    }
};
use songbird::{
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
    events::CoreEvent
};

struct DisconnectHandler {
    ctx: Context
}

#[async_trait]
impl VoiceEventHandler for DisconnectHandler {
    async fn act(&self, vctx: &EventContext<'_>) -> Option<Event> {
        match vctx {
            EventContext::DriverDisconnect(data) => {
                debug!(
                    kind = ?data.kind,
                    reason = ?data.reason,
                    channel_id = ?data.channel_id,
                    "Songbird DriverDisconnect"
                );

                // 正常な切断でない場合は再接続を試みる
                let guild_id = GuildId::from(data.guild_id.0);
                let connected = {
                    let data_read = self.ctx.data.read().await;
                    let connected = data_read.get::<ConnectedChannel>().unwrap();
                    let lock = connected.lock().unwrap();
                    lock.get(&guild_id).cloned()
                };
                if let Some(channel_id) = connected {
                    let manager = songbird::get(&self.ctx).await.unwrap();
                    let (_, result) = manager.join(guild_id, channel_id).await;
                    if let Err(why) = result {
                        error!(channel_id=%channel_id, "Failed to reconnect to voice channel: {why:?}");
                    }
                }
            },
            EventContext::DriverReconnect(data) => {
                debug!(channel_id = ?data.channel_id, "Songbird DriverReconnect");
            },
            _ => {}
        }
        Some(Event::Core(CoreEvent::DriverDisconnect))
    }
}

async fn run_inner(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<impl ToString, impl ToString> {
    let options = &interaction.data.options;

    let guild_id = interaction.guild_id.unwrap();
    let guild = ctx.cache.guild(guild_id).unwrap();

    let channel_id = if let Some(CommandDataOptionValue::Channel(channel)) =
        &options.get(0).and_then(|opt| opt.resolved.as_ref())
    {
        Some(channel.id)
    } else {
        guild.voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    };

    debug!(channel_id = ?channel_id, "/join");

    let Some(connect_to) = channel_id else {
        return Err("接続に失敗しました。");
    };

    let manager = songbird::get(ctx).await.unwrap();
    let (handle, result) = manager.join(guild_id, connect_to).await;
    if result.is_err() {
        return Err("接続に失敗しました。");
    }
    {
        let mut lock = handle.lock().await;
        lock.add_global_event(Event::Core(CoreEvent::DriverDisconnect), DisconnectHandler { ctx: ctx.clone() });
    }

    // メッセージを読むテキストチャンネルを設定する
    {
        let data_read = ctx.data.read().await;
        let channel_id = data_read.get::<TextChannelId>().unwrap();
        let mut lock = channel_id.lock().unwrap();
        lock.insert(guild_id, interaction.channel_id);
        let connected = data_read.get::<ConnectedChannel>().unwrap();
        let mut lock = connected.lock().unwrap();
        lock.insert(guild_id, connect_to);
    }
    

    Ok(format!("<#{connect_to}>に接続しました。"))
}

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let msg = run_inner(ctx, interaction).await;
    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                match msg {
                    Ok(msg) => {
                        message.content(msg)
                    },
                    Err(msg) => {
                        message.ephemeral(true).content(msg)
                    }
                }
            })
    }).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("join")
        .description("ボイスチャンネルに接続します。")
        .create_option(|option| {
            option.name("ボイスチャンネル")
                .description("接続するボイスチャンネル")
                .kind(CommandOptionType::Channel)
                .channel_types(&[ChannelType::Voice])
        })
}
