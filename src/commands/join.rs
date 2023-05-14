use crate::TextChannelId;
use serenity::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::prelude::*;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

async fn run_inner(ctx: &Context, interaction: &ApplicationCommandInteraction) -> impl ToString {
    let guild_id = interaction.guild_id.unwrap();
    let guild = ctx.cache.guild(guild_id).unwrap();

    let channel_id = guild.voice_states
        .get(&interaction.user.id)
        .and_then(|voice_state| voice_state.channel_id);

    let Some(connect_to) = channel_id else {
        return "接続に失敗しました。";
    };

    let manager = songbird::get(ctx).await.unwrap();
    if manager.join(guild_id, connect_to).await.1.is_err() {
        return "接続に失敗しました。";
    }

    // メッセージを読むテキストチャンネルを設定する
    let data_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<TextChannelId>().unwrap().clone()
    };
    
    {
        let mut data = data_lock.write().await;
        data.insert(guild_id, interaction.channel_id);
    }

    "接続しました。"
}

pub async fn run(_options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let msg = run_inner(ctx, interaction).await;
    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content(msg))
    }).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("join").description("ボイスチャンネルに接続します。")
}
