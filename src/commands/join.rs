use crate::TextChannelId;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::prelude::*;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;

pub async fn run(_options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> String {
    let guild_id = interaction.guild_id.unwrap();
    let guild = ctx.cache.guild(guild_id).unwrap();

    let channel_id = guild.voice_states
        .get(&interaction.user.id)
        .and_then(|voice_state| voice_state.channel_id);

    let Some(connect_to) = channel_id else {
        return "接続に失敗しました。".to_string();
    };

    let manager = songbird::get(ctx).await.unwrap();
    if manager.join(guild_id, connect_to).await.1.is_err() {
        return "接続に失敗しました。".to_string();
    }

    // メッセージを読むテキストチャンネルを設定する
    let data_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<TextChannelId>().unwrap().clone()
    };
    
    {
        let mut data = data_lock.write().await;
        data.insert(guild_id, interaction.channel_id);
    }

    "接続しました。".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("join").description("ボイスチャンネルに接続します。")
}
