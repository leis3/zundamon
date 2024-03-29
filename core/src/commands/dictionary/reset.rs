use crate::ConfigData;
use tracing::debug;
use serenity::prelude::*;
use serenity::Result;
use serenity::model::application::{
    component::ButtonStyle,
    interaction::{
        InteractionResponseType,
        application_command::ApplicationCommandInteraction
    }
};

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .content("辞書をリセットしますか？")
                    .components(|component| {
                        component.create_action_row(|action| {
                            action
                                .create_button(|button| {
                                    button.custom_id("reset_cancel").label("キャンセル").style(ButtonStyle::Primary)
                                })
                                .create_button(|button| {
                                    button.custom_id("reset_do").label("リセットする").style(ButtonStyle::Danger)
                                })
                        })
                    })
            })
    }).await?;

    let msg_interaction = interaction
        .get_interaction_response(&ctx.http).await?
        .await_component_interaction(&ctx.shard).await
        .unwrap();
    msg_interaction.defer(&ctx.http).await?;

    debug!(reset = %msg_interaction.data.custom_id, "/dictionary reset");

    let guild_id = interaction.guild_id.unwrap();

    let response_message = match msg_interaction.data.custom_id.as_str() {
        "reset_cancel" => {
            "リセットをキャンセルしました。"
        },
        "reset_do" => {
            let data_read = ctx.data.read().await;
            let config = data_read.get::<ConfigData>().expect("Expected ConfigData in TypeMap.");
            let mut lock = config.lock().unwrap();
            let config = lock.guild_config_mut(guild_id);
            config.dictionary.clear();
            let _ = config.save(guild_id);
            "辞書をリセットしました。"
        },
        _ => unreachable!()
    };

    msg_interaction.edit_original_interaction_response(&ctx.http, |response| {
        response.content(response_message).components(|component| component.set_action_rows(Vec::new()))
    }).await?;

    Ok(())
}
