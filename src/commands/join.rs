use crate::type_map::{TextChannelId, ConnectedChannel};
use crate::debug;
use serenity::prelude::*;
use serenity::builder::CreateApplicationCommand;
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
    if manager.join(guild_id, connect_to).await.1.is_err() {
        return Err("接続に失敗しました。");
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
