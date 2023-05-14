use crate::DictData;
use serenity::prelude::*;
use serenity::Result;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

pub async fn run(_options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let dict = {
        let data_read = ctx.data.read().await;
        data_read.get::<DictData>().expect("Expected DictData in TypeMap.").clone()
    };
    let mut dict = dict.write().await;
    dict.reset();

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content("辞書をリセットしました。"))
    }).await
}
