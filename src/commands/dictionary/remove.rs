use crate::DictData;
use serenity::prelude::*;
use serenity::Result;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
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
    let msg = if dict.remove(key) {
        "辞書から削除しました。"
    } else {
        "指定した単語は辞書に登録されていません。"
    };

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content(msg))
    }).await
}
