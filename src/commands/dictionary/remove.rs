use std::sync::Arc;
use tokio::sync::RwLock;
use crate::dictionary::Dictionary;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;

pub async fn run(options: &[CommandDataOption], dict: Arc<RwLock<Dictionary>>) -> String {
    let CommandDataOptionValue::String(key) = options[0].resolved.as_ref().unwrap() else {
        panic!()
    };

    let mut dict = dict.write().await;
    if dict.remove(key) {
        "辞書から削除しました。".into()
    } else {
        "指定した単語は辞書に登録されていません。".into()
    }
}
