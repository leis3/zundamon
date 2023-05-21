use crate::ConfigData;
use serenity::prelude::*;
use serenity::Result;
use serenity::model::channel::AttachmentType;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::{
        CommandDataOption,
        CommandDataOptionValue,
        ApplicationCommandInteraction
    }
};

pub async fn run(options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let CommandDataOptionValue::String(format) = options[0].resolved.as_ref().unwrap() else {
        panic!()
    };

    let guild_id = interaction.guild_id.unwrap();

    match format.as_str() {
        "JSON" => {
            let config = {
                let data_read = ctx.data.read().await;
                data_read.get::<ConfigData>().expect("Expected ConfigData in TypeMap.").clone()
            };
            let config = config.read().await;
            let dict = &config.0.get(&guild_id).unwrap().dictionary;
            let data = serde_json::to_string_pretty(dict).unwrap();

            interaction.create_interaction_response(&ctx.http, |response| {
                response.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .add_file(AttachmentType::from((data.as_bytes(), "dictionary.json")))
                            .ephemeral(true)
                    })
            }).await
        },
        _ => unreachable!("unsupported file format" )
    }
}
