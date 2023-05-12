use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub fn run(_: &[CommandDataOption]) -> String {
    env!("CARGO_PKG_VERSION").into()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("version").description("バージョンを出力します。")
}
