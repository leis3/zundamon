use crate::ConfigData;
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

    let guild_id = interaction.guild_id.unwrap();

    let msg_interaction = interaction
        .get_interaction_response(&ctx.http).await?
        .await_component_interaction(&ctx.shard).await
        .unwrap();
    msg_interaction.defer(&ctx.http).await?;

    let response_message = match msg_interaction.data.custom_id.as_str() {
        "reset_cancel" => {
            "リセットをキャンセルしました。"
        },
        "reset_do" => {
            {
                let data_read = ctx.data.read().await;
                let config = data_read.get::<ConfigData>().expect("Expected ConfigData in TypeMap.");
                let mut config_lock = config.lock().unwrap();
                config_lock.0.get_mut(&guild_id).unwrap().dictionary.reset();
            }
            "辞書をリセットしました。"
        },
        _ => unreachable!()
    };

    msg_interaction.edit_original_interaction_response(&ctx.http, |response| {
        response.content(response_message).components(|component| component.set_action_rows(Vec::new()))
    }).await?;

    Ok(())
}
