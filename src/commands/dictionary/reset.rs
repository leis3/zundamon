use crate::DictData;
use serenity::prelude::*;
use serenity::Result;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};
use serenity::model::application::component::ButtonStyle;

pub async fn run(_options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
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

    let response_message = match msg_interaction.data.custom_id.as_str() {
        "reset_cancel" => {
            "リセットをキャンセルしました。"
        },
        "reset_do" => {
            let dict = {
                let data_read = ctx.data.read().await;
                data_read.get::<DictData>().expect("Expected DictData in TypeMap.").clone()
            };
            let mut dict = dict.write().await;
            dict.reset();
            "辞書をリセットしました。"
        },
        _ => unreachable!()
    };

    msg_interaction.edit_original_interaction_response(&ctx.http, |response| {
        response.content(response_message).components(|component| component.set_action_rows(Vec::new()))
    }).await?;

    Ok(())
}
