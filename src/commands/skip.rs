use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::prelude::*;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;

pub async fn run(_options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> String {
    let manager = songbird::get(ctx).await.unwrap();

    if let Some(handle) = manager.get(interaction.guild_id.unwrap()) {
        let mut handler = handle.lock().await;

        handler.stop();
    } else {
        return "読み上げのスキップに失敗しました。".into();
    }

    "読み上げをスキップしました".into()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("skip").description("現在の読み上げをスキップします。")
}
