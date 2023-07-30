use crate::ConfigData;
use std::collections::BTreeMap;
use tracing::debug;
use once_cell::sync::Lazy;
use serenity::prelude::*;
use serenity::Result;
use serenity::builder::{
    CreateComponents,
    CreateApplicationCommand
};
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

#[derive(Debug, Clone)]
struct Speaker {
    name: String,
    style: String,
    id: u32
}

static SPEAKERS: Lazy<BTreeMap<String, Vec<Speaker>>> = Lazy::new(|| {
    let Some(speakers) = std::fs::read_to_string("./voicevox_core/model/metas.json").ok()
        .and_then(|metas| serde_json::from_str::<serde_json::Value>(&metas).ok())
        .and_then(|value| value.as_array().cloned())
    else {
        return BTreeMap::new();
    };
    let mut map: BTreeMap<String, Vec<Speaker>> = BTreeMap::new();
    // TODO: 仕様上セレクトメニューは25項目までなのでmetas.jsonのモデルをすべて含めるのを諦めるか
    //       話者選択でのページングを実装する必要がある
    for speaker in speakers.into_iter().take(25) {
        let styles = speaker.get("styles").unwrap().as_array().unwrap();
        for value in styles {
            let name = speaker.get("name").unwrap().as_str().unwrap().to_string();
            let style = value.get("name").unwrap().as_str().unwrap().to_string();
            let id = value.get("id").unwrap().as_u64().unwrap() as u32;
            map.entry(name.clone()).or_default().push(Speaker { name, style, id });
        }
    }
    map
});

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    // 話者名を選択する
    debug!("accept /speaker");
    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message.content("話者を選択してください。")
                    .components(|component| {
                        component.create_action_row(|action| {
                            action.create_select_menu(|menu| {
                                menu.custom_id("speaker_name").options(|opts| {
                                    for name in SPEAKERS.keys() {
                                        opts.create_option(|opt| {
                                            opt.label(name.clone()).value(name.clone())
                                        });
                                    }
                                    opts
                                })
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
    let selected_name = &msg_interaction.data.values[0];

    debug!("selected {}", selected_name);

    // スタイルを選択する
    let message = msg_interaction.edit_original_interaction_response(&ctx.http, |response| {
        response.content("スタイルを選択してください。")
            .components(|component| {
                component.create_action_row(|action| {
                    action.create_select_menu(|menu| {
                        menu.custom_id("speaker_style").options(|opts| {
                            for style in SPEAKERS.get(selected_name).unwrap_or(&Vec::new()) {
                                opts.create_option(|opt| {
                                    opt.label(&style.style).value(style.id)
                                });
                            }
                            opts
                        })
                    })
                })
            })
    }).await?;

    let msg_interaction = message.await_component_interaction(&ctx.shard)
        .await
        .unwrap();
    msg_interaction.defer(&ctx.http).await?;
    let speaker_id: u32 = msg_interaction.data.values[0].parse().unwrap();

    debug!(speaker_id = %speaker_id, "/speaker");

    let guild_id = interaction.guild_id.unwrap();

    {
        let data_read = ctx.data.read().await;
        let config = data_read.get::<ConfigData>().unwrap();
        let mut config_lock = config.lock().unwrap();
        let speaker = &mut config_lock.guild_config_mut(guild_id).speaker_id;
        *speaker = speaker_id;
    }

    let speaker = SPEAKERS[selected_name].iter().find(|s| s.id == speaker_id).unwrap();

    let message_id = msg_interaction.message.id;
    interaction.edit_followup_message(&ctx.http, message_id, |message| {
        message.content(format!("話者を「{}({})」に変更しました。", speaker.name, speaker.style))
            .set_components(CreateComponents::default())
    }).await?;
    Ok(())

    /*
    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message.content(format!("話者を「{}({})」に変更しました。", speaker.name, speaker.style))
            })
    }).await
    */
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let _s = &*SPEAKERS;
    command.name("speaker").description("話者を切り替えます。")
}
