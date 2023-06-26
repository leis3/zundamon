mod add;
mod remove;
mod reset;
mod export;
mod import;

use serenity::prelude::*;
use serenity::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let option = &interaction.data.options[0];

    
    let span = tracing::debug_span!(
        parent: None,
        "ApplicationCommandInteraction",
        guild_id = interaction.guild_id.map(|e| e.0),
        channel_id = interaction.channel_id.0,
        user = format!("{}({})", interaction.user.name, interaction.user.id.0)
    );
    let _enter = span.enter();

    match option.name.as_str() {
        "add" => add::run(ctx, interaction).await,
        "remove" => remove::run(ctx, interaction).await,
        "reset" => reset::run(ctx, interaction).await,
        "export" => export::run(ctx, interaction).await,
        "import" => import::run(ctx, interaction).await,
        _ => panic!("unexpected subcommand name")
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("dictionary")
        .description("...")
        .create_option(|option| {
            option.name("add")
                .description("辞書に単語を登録します。")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option.name("単語")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .description("辞書に登録する単語")
                })
                .create_sub_option(|option| {
                    option.name("読み")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .description("辞書に登録する単語の読み方")
                })
                .create_sub_option(|option| {
                    option.name("正規表現")
                        .kind(CommandOptionType::Boolean)
                        .description("正規表現として登録する場合はTrue")
                })
        })
        .create_option(|option| {
            option.name("remove")
                .description("辞書から単語を削除します。")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option.name("単語")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .description("辞書から削除する単語")
                })
        })
        .create_option(|option| {
            option.name("reset")
                .description("辞書をリセットします。")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option.name("export")
                .description("辞書に登録されている単語をエクスポートします。")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option.name("format")
                        .description("出力するフォーマット")
                        .kind(CommandOptionType::String)
                        .add_string_choice("JSON", "JSON")
                        .required(true)
                })
        })
        .create_option(|option| {
            option.name("import")
                .description("辞書に登録する単語をインポートします。")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option.name("file")
                        .description("インポートするファイル")
                        .kind(CommandOptionType::Attachment)
                        .required(true)
                })
        })
}
