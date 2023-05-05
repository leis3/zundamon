use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::prelude::*;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;

pub async fn run(_options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> String {
    let guild_id = interaction.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap();

    if manager.leave(guild_id).await.is_err() {
        return "切断に失敗しました。".to_string();
    }
    
    "切断しました。".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("leave").description("ボイスチャンネルから切断します。")
}
