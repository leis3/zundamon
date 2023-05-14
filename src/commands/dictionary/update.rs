use std::collections::HashMap;
use crate::DictData;
use crate::dictionary::DictionaryItem;
use serenity::prelude::*;
use serenity::Result;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

pub async fn run(options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let map = options.iter().map(|option| {
        (option.name.as_str(), option.resolved.as_ref().unwrap())
    }).collect::<HashMap<_, _>>();

    let CommandDataOptionValue::String(key) = map["単語"].clone() else { panic!() };
    let CommandDataOptionValue::String(value) = map["読み"].clone() else { panic!() };
    let CommandDataOptionValue::Boolean(is_regex) = *map["正規表現"] else { panic!() };
    let CommandDataOptionValue::Integer(priority) = *map["優先度"] else { panic!() };
    let item = DictionaryItem { key, value, is_regex, priority };

    let dict = {
        let data_read = ctx.data.read().await;
        data_read.get::<DictData>().expect("Expected DictData in TypeMap.").clone()
    };
    let mut dict = dict.write().await;
    let msg = if dict.update(item) {
        "辞書を上書きしました。"
    } else {
        "指定した単語は辞書に登録されていません。"
    };

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content(msg))
    }).await
}
