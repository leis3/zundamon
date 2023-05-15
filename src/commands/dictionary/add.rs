use std::collections::HashMap;
use crate::DictData;
use crate::dictionary::DictionaryItem;
use serenity::prelude::*;
use serenity::Result;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::{
        CommandDataOption,
        CommandDataOptionValue,
        ApplicationCommandInteraction
    }
};

pub async fn run(options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let map = options.iter().map(|option| {
        (option.name.as_str(), option.resolved.as_ref().unwrap())
    }).collect::<HashMap<_, _>>();

    let CommandDataOptionValue::String(key) = map["単語"].clone() else { panic!() };
    let CommandDataOptionValue::String(value) = map["読み"].clone() else { panic!() };
    let CommandDataOptionValue::Boolean(is_regex) = **map.get("正規表現").unwrap_or(&&CommandDataOptionValue::Boolean(false)) else { panic!() };
    let CommandDataOptionValue::Integer(priority) = **map.get("優先度").unwrap_or(&&CommandDataOptionValue::Integer(0)) else { panic!() };
    let item = DictionaryItem { key, value, is_regex, priority };

    let dict = {
        let data_read = ctx.data.read().await;
        data_read.get::<DictData>().expect("Expected DictData in TypeMap.").clone()
    };
    let mut dict = dict.write().await;
    dict.add(item);

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content("辞書に登録しました。"))
    }).await
}
