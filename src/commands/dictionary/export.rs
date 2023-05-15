use crate::DictData;
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

    match format.as_str() {
        "JSON" => {
            let dict = {
                let data_read = ctx.data.read().await;
                data_read.get::<DictData>().expect("Expected DictData in TypeMap.").clone()
            };
            let data = serde_json::to_string_pretty(&*dict.read().await.items).unwrap();

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
