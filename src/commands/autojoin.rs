use crate::ConfigData;
use crate::debug;
use serenity::prelude::*;
use serenity::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::{
        CommandDataOptionValue,
        ApplicationCommandInteraction
    }
};

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let options = &interaction.data.options;
    let CommandDataOptionValue::Boolean(enable) = *options[0].resolved.as_ref().unwrap() else {
        panic!()
    };

    debug!(enable = %enable, "/autojoin");

    let guild_id = interaction.guild_id.unwrap();

    {
        let data_read = ctx.data.read().await;
        let config = data_read.get::<ConfigData>().expect("Expected ConfigData in TypeMap.");
        let mut config_lock = config.lock().unwrap();
        let autojoin = &mut config_lock.guild_config_mut(guild_id).autojoin;
        *autojoin = enable;
    }

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message.content(if enable {"自動入室を有効にしました。"} else {"自動入室を無効にしました。"})
            })
    }).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("autojoin")
        .description("自動入室機能を切り替えます。")
        .create_option(|option| {
            option.name("enable")
                .description("自動入室機能を有効にします。")
                .kind(CommandOptionType::Boolean)
                .required(true)
        })
}
