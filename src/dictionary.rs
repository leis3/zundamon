use crate::config::{CONFIG_DIR, CONFIG_FILE};
use std::path::Path;
use std::io::Result;
use serde::{Serialize, Deserialize};
use serenity::model::prelude::GuildId;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dictionary {
    pub items: Vec<DictionaryItem>
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryItem {
    pub key: String,
    pub value: String,
    /// 正規表現かどうか
    pub is_regex: bool,
    /// 辞書変換適用の優先度(値が大きいほど優先度が高い)
    pub priority: i64
}

impl Dictionary {
    pub fn load(guild_id: GuildId) -> Result<Self> {
        let path = Path::new(CONFIG_DIR).join(CONFIG_FILE).join(guild_id.0.to_string());
        let content = std::fs::read_to_string(path)?;
        let mut dic: Dictionary = serde_json::from_str(&content)?;
        dic.items.sort_unstable_by_key(|item| item.priority);
        Ok(dic)
    }

    pub fn apply(&self, text: &str) -> String {
        let mut text = text.to_owned();

        for item in &self.items {
            if item.is_regex {
                let re = regex::Regex::new(&item.key).unwrap();
                text = re.replace_all(&text, &item.value).into_owned();
            } else {
                text = text.replace(&item.key, &item.value);
            }
        }

        text
    }

    fn position(&self, key: &str) -> Option<usize> {
        self.items.iter().position(|_item| _item.key == key)
    }
    
    /// return true if updated
    pub fn add(&mut self, item: DictionaryItem) -> bool {
        let updated = if let Some(position) = self.position(&item.key) {
            self.items[position] = item;
            true
        }
        else {
            self.items.push(item);
            false
        };
        self.items.sort_unstable_by_key(|item| item.priority);
        updated
    }

    /// return true if removed
    pub fn remove(&mut self, key: &str) -> bool {
        if let Some(position) = self.position(key) {
            self.items.remove(position);
            true
        } else {
            false
        }
    }

    /// return true if updated
    pub fn update(&mut self, item: DictionaryItem) -> bool {
        if let Some(position) = self.position(&item.key) {
            self.items[position] = item;
            self.items.sort_unstable_by_key(|item| item.priority);
            true
        } else {
            false
        }
    }
    
    pub fn reset(&mut self) {
        self.items.clear();
    }

    pub fn import(&mut self, dict: &Dictionary, overwrite: bool) {
        for item in &dict.items {
            if let (Some(position), true) = (self.position(&item.key), overwrite) {
                self.items[position] = item.clone();
            } else {
                self.items.push(item.clone());
            }
        }
        self.items.sort_unstable_by_key(|item| item.priority);
    }
}
