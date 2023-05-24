use std::collections::HashMap;
use crate::ConfigData;
use crate::dictionary::DictionaryItem;
use serenity::prelude::*;
use serenity::Result;
use serenity::utils::Color;
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

    if is_regex && regex::Regex::new(&key).is_err() {
        let msg = "入力した正規表現が無効です。";
        return interaction.create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.ephemeral(true).content(msg)
                })
        }).await;
    }

    let item = DictionaryItem { key, value, is_regex, priority };

    let guild_id = interaction.guild_id.unwrap();

    let updated = {
        let data_read = ctx.data.read().await;
        let config = data_read.get::<ConfigData>().expect("Expected ConfigData in TypeMap.");
        let mut config_lock = config.lock().unwrap();
        config_lock.0.get_mut(&guild_id).unwrap().dictionary.add(item.clone())
    };

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message.embed(|embed| {
                    let title = if updated {
                        "辞書を上書きしました。"
                    } else {
                        "辞書に登録しました。"
                    };
                    embed.title(title)
                        .color(Color::from_rgb(0x66, 0xbb, 0x6a))
                        .fields([
                            ("単語", format!("```{}```", item.key), false),
                            ("読み", format!("```{}```", item.value), false)
                        ])
                })
            })
    }).await
}
