use crate::ConfigData;
use tracing::debug;
use serenity::prelude::*;
use serenity::Result;
use serenity::model::channel::AttachmentType;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::{
        CommandDataOptionValue,
        ApplicationCommandInteraction
    }
};

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let options = &interaction.data.options[0].options;
    let CommandDataOptionValue::String(format) = options[0].resolved.as_ref().unwrap() else {
        panic!()
    };

    debug!(format = %format, "/dictionary export");

    let guild_id = interaction.guild_id.unwrap();

    match format.as_str() {
        "JSON" => {
            let data = {
                let data_read = ctx.data.read().await;
                let config = data_read.get::<ConfigData>().expect("Expected ConfigData in TypeMap.");
                let mut lock = config.lock().unwrap();
                let dict = &lock.guild_config(guild_id).dictionary;
                serde_json::to_string_pretty(dict).unwrap()
            };

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
