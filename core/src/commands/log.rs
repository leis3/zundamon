use std::path::Path;
use tracing::debug;
use serenity::prelude::*;
use serenity::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::channel::AttachmentType;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::{
        CommandDataOptionValue,
        ApplicationCommandInteraction
    }
};

use crate::config::GlobalConfig;

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let admins = GlobalConfig::load().map(|config| config.admin_user).unwrap_or_default();
    if !admins.contains(&interaction.user.id) {
        return interaction.create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.ephemeral(true).content("このコマンドを実行する権限がありません。")
                })
        }).await;
    }


    let options = &interaction.data.options;
    let CommandDataOptionValue::String(date) = options[0].resolved.as_ref().unwrap() else {
        panic!()
    };

    debug!(date = %date, "/log");

    if !regex::Regex::new("^[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$").unwrap().is_match(date) {
        return interaction.create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.ephemeral(true).content("日付の形式が無効です。yyyy-mm--dd形式で指定してください。")
                })
        }).await;
    }

    let file_path = Path::new("logs").join(format!("{date}.log"));
    if !file_path.exists() {
        return interaction.create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.ephemeral(true).content("指定した日付のログファイルが見つかりませんでした。")
                })
        }).await;
    }

    let file = tokio::fs::File::open(&file_path).await?;

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .ephemeral(true)
                    .add_file(AttachmentType::File { file: &file, filename: format!("{date}.log") })
            })
    }).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("log")
        .description("ログファイルを出力します。")
        .create_option(|option| {
            option.name("date")
                .description("ログの日付 (yyyy-mm-dd形式)")
                .kind(CommandOptionType::String)
        })
}
