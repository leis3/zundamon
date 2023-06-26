use tracing::debug;
use serenity::Result;
use serenity::prelude::*;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    debug!("/skip");
    
    let manager = songbird::get(ctx).await.unwrap();

    let success = if let Some(handle) = manager.get(interaction.guild_id.unwrap()) {
        let mut handler = handle.lock().await;
        handler.stop();
        true
    } else {
        false
    };

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                let msg = if success {
                    "読み上げをスキップしました。"
                } else {
                    "読み上げのスキップに失敗しました。"
                };
                message.ephemeral(!success).content(msg)
            })
    }).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("skip").description("現在の読み上げをスキップします。")
}
