use crate::DictData;
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
    let CommandDataOptionValue::String(key) = options[0].resolved.as_ref().unwrap() else {
        panic!()
    };

    let dict = {
        let data_read = ctx.data.read().await;
        data_read.get::<DictData>().expect("Expected DictData in TypeMap.").clone()
    };
    let mut dict = dict.write().await;
    let removed = dict.remove(key);

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                if removed {
                    message.embed(|embed| {
                        embed.title("辞書から削除しました。")
                            .color(Color::from_rgb(0x66, 0xbb, 0x6a))
                            .field("単語", format!("`{}`", key), false)
                    })
                } else {
                    message.ephemeral(true).content("指定した単語は辞書に登録されていません。")
                }
            })
    }).await
}
