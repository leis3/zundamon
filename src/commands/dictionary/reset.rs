use std::sync::Arc;
use tokio::sync::RwLock;
use crate::dictionary::Dictionary;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub async fn run(_options: &[CommandDataOption], dict: Arc<RwLock<Dictionary>>) -> String {
    let mut dict = dict.write().await;
    dict.reset();

    "辞書をリセットしました。".into()
}
