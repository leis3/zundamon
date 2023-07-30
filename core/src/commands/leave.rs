use crate::type_map::ConnectedChannel;
use tracing::debug;
use serenity::Result;
use serenity::prelude::*;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    debug!("/leave");

    let guild_id = interaction.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap();

    {
        let data_read = ctx.data.read().await;
        let connected = data_read.get::<ConnectedChannel>().unwrap();
        let mut lock = connected.lock().unwrap();
        lock.remove(&guild_id);
    }
    let success =  manager.leave(guild_id).await.is_ok();

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                let msg = if success {
                    "切断しました。"
                } else {
                    
                    "切断に失敗しました。"
                };
                message.ephemeral(!success).content(msg)
            })
    }).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("leave").description("ボイスチャンネルから切断します。")
}
