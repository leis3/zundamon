use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::dictionary::{Dictionary, DictionaryItem};
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;

pub async fn run(options: &[CommandDataOption], dict: Arc<RwLock<Dictionary>>) -> String {
    let map = options.iter().map(|option| {
        (option.name.as_str(), option.resolved.as_ref().unwrap())
    }).collect::<HashMap<_, _>>();

    let CommandDataOptionValue::String(key) = map["単語"].clone() else { panic!() };
    let CommandDataOptionValue::String(value) = map["読み"].clone() else { panic!() };
    let CommandDataOptionValue::Boolean(is_regex) = *map["正規表現"] else { panic!() };
    let CommandDataOptionValue::Integer(priority) = *map["優先度"] else { panic!() };
    let item = DictionaryItem { key, value, is_regex, priority };

    let mut dict = dict.write().await;
    dict.add(item);

    "辞書に登録しました。".into()
}
