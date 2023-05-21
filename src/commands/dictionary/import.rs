use std::collections::HashMap;
use crate::ConfigData;
use crate::dictionary::Dictionary;
use serenity::prelude::*;
use serenity::utils::Color;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::{
        CommandDataOption,
        CommandDataOptionValue,
        ApplicationCommandInteraction
    }
};

async fn run_inner(options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<impl ToString, impl ToString> {
    let map = options.iter().map(|option| {
        (option.name.as_str(), option.resolved.as_ref().unwrap())
    }).collect::<HashMap<_, _>>();

    let CommandDataOptionValue::Attachment(attachment) = map["attachment"] else { panic!() };

    let format = attachment.filename.as_str();
    if !["json"].contains(&format) {
        return Err("サポートされていないファイル形式です。");
    }
    let CommandDataOptionValue::Boolean(overwrite) = *map["overwrite"] else { panic!() };

    let Ok(response) = reqwest::get(&attachment.url).await else {
        return Err("ファイルの取得に失敗しました。");
    };

    let guild_id = interaction.guild_id.unwrap();

    match format {
        "json" => {
            let Ok(new_dict) = response.json::<Dictionary>().await else {
                return Err("無効なJSONデータです。");
            };
            let config = {
                let data_read = ctx.data.read().await;
                data_read.get::<ConfigData>().expect("Expected ConfigData in TypeMap.").clone()
            };
            let mut dict = config.write().await;
            dict.0.get_mut(&guild_id).unwrap().dictionary.import(&new_dict, overwrite);
            Ok("辞書をインポートしました。")
        },
        _ => unreachable!("unsupported file format")
    }
}

pub async fn run(_options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let msg = run_inner(_options, ctx, interaction).await;
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
