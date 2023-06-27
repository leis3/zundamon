use crate::config::{CONFIG_DIR, DICT_FILE};
use std::path::Path;
use std::collections::HashSet;
use serenity::model::prelude::GuildId;
use aho_corasick::AhoCorasick;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Dictionary {
    items: Vec<DictItem>,
    regex_items: Vec<DictItem>,
    automaton: AhoCorasick,
    guild_id: Option<GuildId>,
    keys: HashSet<String>
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictItem {
    pub key: String,
    pub value: String,
    pub is_regex: bool
}

impl Dictionary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_items(items: Vec<DictItem>, guild_id: Option<GuildId>) -> Result<Self> {
        let (regex_items, items): (Vec<DictItem>, Vec<DictItem>) = items.into_iter().partition(|item| item.is_regex);
        let automaton = AhoCorasick::new(items.iter().map(|item| item.key.clone()))?;
        let keys = regex_items.iter().chain(items.iter()).map(|item| item.key.clone()).collect();
        Ok(Self { items, regex_items, automaton, guild_id, keys })
    }

    pub fn load(guild_id: GuildId) -> Result<Self> {
        let path = Path::new(CONFIG_DIR).join(guild_id.0.to_string()).join(DICT_FILE);
        if let Ok(data) = std::fs::read_to_string(&path) {
            let dict: Vec<DictItem> = serde_json::from_str(&data)?;
            Self::from_items(dict, Some(guild_id))
        } else {
            std::fs::create_dir_all(path.parent().unwrap())?;
            let mut file = std::fs::File::create(path)?;
            writeln!(file, "[]")?;
            Ok(Self::new())
        }
    }

    pub fn save(&self) -> Result<()> {
        let Some(guild_id) = self.guild_id else {
            anyhow::bail!("expected guild_id value");
        };
        let dict = self.items.iter().chain(self.regex_items.iter()).collect::<Vec<_>>();
        let path = Path::new(CONFIG_DIR).join(guild_id.0.to_string()).join(DICT_FILE);
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "{}", serde_json::to_string_pretty(&dict)?)?;
        Ok(())
    }

    pub fn apply(&self, text: &str) -> Result<String> {
        let mut text = text.to_owned();

        for item in &self.regex_items {
            let re = regex::Regex::new(&item.key)?;
            text = re.replace_all(&text, &item.value).into_owned();
        }
        
        // インデックスのずれを防ぐため後ろから変換
        let mut text_bytes = text.as_bytes().to_vec();
        let v = self.automaton.find_iter(&text).collect::<Vec<_>>();
        for mat in v.into_iter().rev() {
            let index = mat.pattern().as_usize();
            let value = &self.items[index].value;
            text_bytes.splice(mat.start()..mat.end(), value.bytes());
        }
        
        Ok(String::from_utf8(text_bytes)?)
    }

    pub fn add(&mut self, item: DictItem) -> Result<bool> {
        let mut is_updated = false;
        if self.keys.contains(&item.key) {
            self.remove(&item.key)?;
            is_updated = true;
        }
        self.keys.insert(item.key.clone());
        if item.is_regex {
            self.regex_items.push(item);
        } else {
            self.items.push(item);
            self.automaton = AhoCorasick::new(self.items.iter().map(|item| item.key.clone()))?;
        }
        Ok(is_updated)
    }

    pub fn add_items(&mut self, items: Vec<DictItem>) -> Result<()> {
        for item in items {
            self.keys.insert(item.key.clone());
            if item.is_regex {
                if let Some(old_item) = self.regex_items.iter_mut().find(|i| i.key == item.key) {
                    *old_item = item;
                } else {
                    self.regex_items.push(item);
                }
            } else {
                if let Some(old_item) = self.items.iter_mut().find(|i| i.key == item.key) {
                    *old_item = item;
                } else {
                    self.items.push(item);
                }
            }
        }
        self.automaton = AhoCorasick::new(self.items.iter().map(|item| item.key.clone()))?;
        Ok(())
    }

    pub fn remove(&mut self, key: &str) -> Result<bool> {
        if !self.keys.contains(key) {
            return Ok(false);
        }
        self.keys.remove(key);
        if let Some(index) = self.items.iter().position(|item| item.key == key) {
            self.items.remove(index);
            self.automaton = AhoCorasick::new(self.items.iter().map(|item| item.key.clone()))?;
        } else if let Some(index) = self.regex_items.iter().position(|item| item.key == key) {
            self.regex_items.remove(index);
        }
        Ok(true)
    }

    pub fn reset(&mut self) -> Result<()> {
        self.items.clear();
        self.regex_items.clear();
        self.keys.clear();
        self.automaton = AhoCorasick::new(self.items.iter().map(|item| item.key.clone()))?;
        Ok(())
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            regex_items: Vec::new(),
            automaton: AhoCorasick::new(Vec::<String>::new()).unwrap(),
            guild_id: None,
            keys: HashSet::new()
        }
    }
}

impl Serialize for Dictionary {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let items = self.items.iter().chain(self.regex_items.iter()).collect::<Vec<_>>();
        items.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use crate::dictionary::*;

    #[test]
    fn test_from_items() {
        let items = vec![
            DictItem {
                key: "foo".to_string(),
                value: "bar".to_string(),
                is_regex: false
            },
            DictItem {
                key: "baz".to_string(),
                value: "qux".to_string(),
                is_regex: true
            }
        ];
        let dict = Dictionary::from_items(items, None).unwrap();
        assert_eq!(dict.items.len(), 1);
        assert_eq!(dict.regex_items.len(), 1);
    }

    #[test]
    fn test_apply() {
        let items = vec![
            DictItem {
                key: "foo".into(),
                value: "bar".into(),
                is_regex: false
            },
            DictItem {
                key: "ba[rz]".into(),
                value: "qux".into(),
                is_regex: true
            }
        ];
        let dict = Dictionary::from_items(items, None).unwrap();
        assert_eq!(dict.apply("foobarbaz").unwrap(), "barquxqux".to_string());
    }

    #[test]
    fn test_add_remove_reset() {
        let mut dict = Dictionary::new();

        let item = DictItem {
            key: "test".to_string(),
            value: "replacement".to_string(),
            is_regex: false
        };

        dict.add(item.clone()).unwrap();
        assert_eq!(dict.items.len(), 1);
        assert_eq!(dict.items[0], item);
        assert!(dict.regex_items.is_empty());

        dict.remove("test").unwrap();
        assert!(dict.items.is_empty());
        assert!(dict.regex_items.is_empty());
        
        dict.add(item.clone()).unwrap();
        dict.reset().unwrap();
        assert!(dict.items.is_empty());
        assert!(dict.regex_items.is_empty());
    }
}
