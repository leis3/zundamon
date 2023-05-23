use crate::ConfigData;
use serenity::prelude::*;
use serenity::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::application::interaction::{
    InteractionResponseType,
    application_command::{
        CommandDataOption,
        CommandDataOptionValue,
        ApplicationCommandInteraction
    }
};

pub async fn run(options: &[CommandDataOption], ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let CommandDataOptionValue::Boolean(enable) = *options[0].resolved.as_ref().unwrap() else {
        panic!()
    };

    let guild_id = interaction.guild_id.unwrap();

    {
        let data_read = ctx.data.read().await;
        let config = data_read.get::<ConfigData>().expect("Expected ConfigData in TypeMap.");
        let mut config_lock = config.lock().unwrap();
        config_lock.0.get_mut(&guild_id).unwrap().time_signal = enable;
    }

    interaction.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message.content(if enable {"時報機能を有効にしました。"} else {"時報機能を無効にしました。"})
            })
    }).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("time-signal")
        .description("時報機能を切り替えます。")
        .create_option(|option| {
            option.name("enable")
                .description("時報機能を有効にします。")
                .kind(CommandOptionType::Boolean)
                .required(true)
        })
}
