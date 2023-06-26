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

    debug!(key = %key, "/dictionary remove");

    let guild_id = interaction.guild_id.unwrap();

    let is_removed = {
        let data_read = ctx.data.read().await;
        let config = data_read.get::<ConfigData>().expect("Expected ConfigData in TypeMap.");
        let mut config_lock = config.lock().unwrap();
        let dict = &mut config_lock.guild_config_mut(guild_id).dictionary;
        let is_removed = dict.remove(key);
        let _ = dict.save();
        is_removed
    };

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                if matches!(is_removed, Ok(true)) {
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
