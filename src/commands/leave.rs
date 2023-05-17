use serenity::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::prelude::*;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

pub async fn run(_options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap();

    let success =  manager.leave(guild_id).await.is_ok();

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                let msg = if success {
                    "切断に失敗しました。"
                } else {
                    "切断しました。"
                };
                message.ephemeral(!success).content(msg)
            })
    }).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("leave").description("ボイスチャンネルから切断します。")
}
