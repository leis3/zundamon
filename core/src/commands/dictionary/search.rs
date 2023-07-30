use crate::ConfigData;
use tracing::debug;
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
    let CommandDataOptionValue::String(key) = options[0].resolved.as_ref().unwrap() else {
        panic!()
    };

    debug!(key = %key, "/dictionary search");

    let guild_id = interaction.guild_id.unwrap();

    let item = {
        let data_read = ctx.data.read().await;
        let config = data_read.get::<ConfigData>().expect("Expected ConfigData in TypeMap.");
        let mut lock = config.lock().unwrap();
        let dict = &lock.guild_config(guild_id).dictionary;
        dict.get(key).cloned()
    };

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                if let Some(item) = item {
                    message.ephemeral(true)
                        .embed(|embed| {
                            embed.title("以下のように登録されています。")
                                .description(format!("正規表現: {}", if item.is_regex {"あり"} else {"なし"}))
                                .color(Color::from_rgb(0x66, 0xbb, 0x6a))
                                .field("単語", format!("```{}```", item.key), false)
                                .field("読み", format!("```{}```", item.value), false)
                        })
                } else {
                    message.ephemeral(true)
                        .content("指定した単語は登録されていません。")
                }
            })
    }).await
}
