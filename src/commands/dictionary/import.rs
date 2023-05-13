use std::sync::Arc;
use std::collections::HashMap;
use std::unreachable;
use tokio::sync::RwLock;
use crate::dictionary::Dictionary;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;

pub async fn run(options: &[CommandDataOption], dict: Arc<RwLock<Dictionary>>) -> String {
    let map = options.iter().map(|option| {
        (option.name.as_str(), option.resolved.as_ref().unwrap())
    }).collect::<HashMap<_, _>>();

    let CommandDataOptionValue::Attachment(attachment) = map["attachment"] else { panic!() };

    let format = attachment.filename.as_str();
    if !["json"].contains(&format) {
        return "サポートされていないファイル形式です。".into();
    }
    let CommandDataOptionValue::Boolean(overwrite) = *map["overwrite"] else { panic!() };

    let Ok(response) = reqwest::get(&attachment.url).await else {
        return "ファイルの取得に失敗しました。".into();
    };

    match format {
        "json" => {
            let Ok(new_dict) = response.json::<Dictionary>().await else {
                return "無効なJSONデータです。".into();
            };
            let mut dict = dict.write().await;
            dict.import(&new_dict, overwrite);
            "辞書をインポートしました。".into()
        },
        _ => unreachable!("unsupported file format")
    }
}
