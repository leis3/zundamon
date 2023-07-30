use std::collections::HashMap;
use crate::ConfigData;
use dictionary::DictItem;
use tracing::debug;
use serenity::prelude::*;
use serenity::utils::Color;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::{
        CommandDataOptionValue,
        ApplicationCommandInteraction
    }
};

async fn run_inner(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<impl ToString, impl ToString> {
    let options = &interaction.data.options[0].options;
    let map = options.iter().map(|option| {
        (option.name.as_str(), option.resolved.as_ref().unwrap())
    }).collect::<HashMap<_, _>>();

    let CommandDataOptionValue::Attachment(file) = map["file"] else { panic!() };

    debug!(file = %file.filename, "/dictionary import");

    let path = std::path::Path::new(&file.filename);
    let Some(format) = path.extension().and_then(|f| f.to_str()) else {
        return Err("ファイル形式が不明です。適切な拡張子を付けて再度実行してください。");
    };
    if !["json"].contains(&format) {
        return Err("サポートされていないファイル形式です。");
    }

    let Ok(response) = reqwest::get(&file.url).await else {
        return Err("ファイルの取得に失敗しました。");
    };

    let guild_id = interaction.guild_id.unwrap();

    match format {
        "json" => {
            let Ok(items) = response.json::<Vec<DictItem>>().await else {
                return Err("無効なJSONデータです。");
            };
            {
                let data_read = ctx.data.read().await;
                let config = data_read.get::<ConfigData>().expect("Expected ConfigData in TypeMap.");
                let mut lock = config.lock().unwrap();
                let config = lock.guild_config_mut(guild_id);
                config.dictionary.extend(items);
                let _ = config.save(guild_id);
            }
            Ok("辞書をインポートしました。")
        },
        _ => unreachable!("unsupported file format")
    }
}

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let msg = run_inner(ctx, interaction).await;
    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                match msg {
                    Ok(msg) => {
                        message.embed(|embed| {
                            embed.title(msg)
                                .color(Color::from_rgb(0x66, 0xbb, 0x6a))
                        })
                    },
                    Err(msg) => {
                        message.ephemeral(true).content(msg)
                    }
                }
            })
    }).await
}
