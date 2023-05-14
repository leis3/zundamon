use serenity::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::prelude::*;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

pub async fn run(_options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let manager = songbird::get(ctx).await.unwrap();

    let msg = if let Some(handle) = manager.get(interaction.guild_id.unwrap()) {
        let mut handler = handle.lock().await;
        handler.stop();
        "読み上げをスキップしました。"
    } else {
        "読み上げのスキップに失敗しました。"
    };

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content(msg))
    }).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("skip").description("現在の読み上げをスキップします。")
}
