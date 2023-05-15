mod add;
mod remove;
mod update;
mod reset;
mod export;
mod import;

use serenity::Result;
use serenity::prelude::*;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;

pub async fn run(options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let option = &options[0];

    match option.name.as_str() {
        "add" => add::run(&option.options, ctx, interaction).await,
        "remove" => remove::run(&option.options, ctx, interaction).await,
        "update" => update::run(&option.options, ctx, interaction).await,
        "reset" => reset::run(&option.options, ctx, interaction).await,
        "export" => export::run(&option.options, ctx, interaction).await,
        "import" => import::run(&option.options, ctx, interaction).await,
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
                .create_sub_option(|option| {
                    option.name("優先度")
                        .kind(CommandOptionType::Integer)
                        .min_int_value(0)
                        .max_int_value(255)
                        .description("辞書変換を適用する優先度(数値が高いほど優先度が高い)")
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
            option.name("update")
                .description("辞書に登録されている単語を更新します。")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option.name("単語")
                        .description("辞書に登録されている単語")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option.name("読み")
                        .kind(CommandOptionType::String)
                        .description("辞書に登録する単語の読み方")
                })
                .create_sub_option(|option| {
                    option.name("正規表現")
                        .kind(CommandOptionType::Boolean)
                        .description("正規表現として登録する場合はTrue")
                })
                .create_sub_option(|option| {
                    option.name("優先度")
                        .kind(CommandOptionType::Integer)
                        .min_int_value(0)
                        .max_int_value(255)
                        .description("辞書変換を適用する優先度(数値が高いほど優先度が高い)")
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
                .create_sub_option(|option| {
                    option.name("overwrite")
                        .description("既に登録されている単語を上書きする場合はTrue")
                        .kind(CommandOptionType::Boolean)
                })
        })
}
