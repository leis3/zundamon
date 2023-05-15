use std::path::Path;
use std::io::{Result, Write};
use serde::{Serialize, Deserialize};

const DIR: &str = "data";
const FILENAME: &str = "dictionary.json";

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
    pub fn new() -> Self {
        if let Ok(dict) = Self::load() {
            dict
        } else {
            Self::default()
        }
    }

    pub fn load() -> Result<Self> {
        let content = std::fs::read_to_string(Path::new(DIR).join(FILENAME))?;
        let mut dic: Dictionary = serde_json::from_str(&content)?;
        dic.items.sort_unstable_by_key(|item| item.priority);
        Ok(dic)
    }

    pub fn save(&self) -> Result<()> {
        if !Path::new(DIR).exists() {
            std::fs::create_dir(DIR)?;
        }
        let mut file = std::fs::File::create(Path::new(DIR).join(FILENAME))?;
        writeln!(file, "{}", serde_json::to_string_pretty(self)?)
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
    
    pub fn add(&mut self, item: DictionaryItem) {
        if let Some(position) = self.position(&item.key) {
            self.items[position] = item;
        }
        else {
            self.items.push(item);
        }
        self.items.sort_unstable_by_key(|item| item.priority);
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
