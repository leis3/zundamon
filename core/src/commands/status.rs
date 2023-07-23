use crate::debug;
use serenity::Result;
use serenity::prelude::*;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    debug!("/status");

    let manager = songbird::get(&ctx).await.unwrap();
    let call = interaction.guild_id.and_then(|id| manager.get(id));
    let channel = {
        if let Some(call) = call {
            let lock = call.lock().await;
            lock.current_channel()
        } else {
            None
        }
    };
    let msg = if let Some(channel_id) = channel {
        format!("<#{channel_id}>に接続されています。")
    } else {
        "現在接続されているチャンネルはありません。".to_string()
    };

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content(msg))
    }).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("status").description("現在接続されているチャンネルを出力します。")
}
