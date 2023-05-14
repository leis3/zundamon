use serenity::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::prelude::*;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

pub async fn run(_options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content(env!("CARGO_PKG_VERSION")))
    }).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("version").description("バージョンを出力します。")
}
