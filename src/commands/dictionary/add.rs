use std::collections::HashMap;
use crate::ConfigData;
use crate::dictionary::DictItem;
use serenity::prelude::*;
use serenity::Result;
use serenity::utils::Color;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::{
        CommandDataOptionValue,
        ApplicationCommandInteraction
    }
};

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let options = &interaction.data.options[0].options;
    let map = options.iter().map(|option| {
        (option.name.as_str(), option.resolved.as_ref().unwrap())
    }).collect::<HashMap<_, _>>();

    let CommandDataOptionValue::String(key) = map["単語"].clone() else { panic!() };
    let CommandDataOptionValue::String(value) = map["読み"].clone() else { panic!() };
    let CommandDataOptionValue::Boolean(is_regex) = **map.get("正規表現").unwrap_or(&&CommandDataOptionValue::Boolean(false)) else { panic!() };

    if is_regex && regex::Regex::new(&key).is_err() {
        let msg = "入力した正規表現が無効です。";
        return interaction.create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.ephemeral(true).content(msg)
                })
        }).await;
    }

    let item = DictItem { key, value, is_regex };

    let guild_id = interaction.guild_id.unwrap();

    let is_updated = {
        let data_read = ctx.data.read().await;
        let config = data_read.get::<ConfigData>().expect("Expected ConfigData in TypeMap.");
        let mut config_lock = config.lock().unwrap();
        let dict = &mut config_lock.guild_config_mut(guild_id).dictionary;
        dict.add(item.clone())
    };

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                if let Ok(is_updated) = is_updated {
                    message.embed(|embed| {
                        let title = if is_updated {
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
                } else {
                    message.ephemeral(true).content("辞書の登録に失敗しました。")
                }
            })
    }).await
}
