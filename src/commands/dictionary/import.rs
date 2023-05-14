use std::collections::HashMap;
use crate::DictData;
use crate::dictionary::Dictionary;
use serenity::prelude::*;
use serenity::Result;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

async fn run_inner(options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> impl ToString {
    let map = options.iter().map(|option| {
        (option.name.as_str(), option.resolved.as_ref().unwrap())
    }).collect::<HashMap<_, _>>();

    let CommandDataOptionValue::Attachment(attachment) = map["attachment"] else { panic!() };

    let format = attachment.filename.as_str();
    if !["json"].contains(&format) {
        return "サポートされていないファイル形式です。";
    }
    let CommandDataOptionValue::Boolean(overwrite) = *map["overwrite"] else { panic!() };

    let Ok(response) = reqwest::get(&attachment.url).await else {
        return "ファイルの取得に失敗しました。";
    };

    match format {
        "json" => {
            let Ok(new_dict) = response.json::<Dictionary>().await else {
                return "無効なJSONデータです。";
            };
            let dict = {
                let data_read = ctx.data.read().await;
                data_read.get::<DictData>().expect("Expected DictData in TypeMap.").clone()
            };
            let mut dict = dict.write().await;
            dict.import(&new_dict, overwrite);
            "辞書をインポートしました。"
        },
        _ => unreachable!("unsupported file format")
    }
}

pub async fn run(_options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let msg = run_inner(_options, ctx, interaction).await;
    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content(msg))
    }).await
}
