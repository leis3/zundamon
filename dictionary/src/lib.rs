use std::path::Path;
use std::collections::HashSet;
use anyhow::Result;
use aho_corasick::AhoCorasick;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct Dictionary {
    items: Vec<DictItem>,
    regex_items: Vec<DictItem>,
    automaton: AhoCorasick,
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

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let items: Vec<DictItem> = serde_json::from_str(&data)?;
        Ok(Self::from_iter(items))
    }

    pub fn from_items(items: Vec<DictItem>) -> Self {
        Self::from_iter(items)
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn insert(&mut self, item: DictItem) -> Option<DictItem> {
        let old = self.remove(&item.key);
        self.keys.insert(item.key.clone());
        if item.is_regex {
            self.regex_items.push(item);
        } else {
            self.items.push(item);
            self.automaton = AhoCorasick::new(self.items.iter().map(|item| item.key.clone())).unwrap();
        }
        old
    }

    pub fn remove(&mut self, key: &str) -> Option<DictItem> {
        if !self.keys.remove(key) {
            return None;
        }
        if let Some(index) = self.items.iter().position(|item| item.key == key) {
            let item = self.items.remove(index);
            self.automaton = AhoCorasick::new(self.items.iter().map(|item| item.key.clone())).unwrap();
            Some(item)
        } else if let Some(index) = self.regex_items.iter().position(|item| item.key == key) {
            Some(self.regex_items.remove(index))
        } else {
            None
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.keys.contains(key)
    }

    pub fn get(&self, key: &str) -> Option<&DictItem> {
        if !self.contains(key) {
            return None;
        }
        self.items.iter().find(|item| item.key == key).or(
            self.regex_items.iter().find(|item| item.key == key)
        )
    }

    pub fn apply<T: Into<String>>(&self, text: T) -> Result<String> {
        let mut text = text.into();

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

        // 絵文字変換
        let text = {
            let mut s = String::new();
            for c in String::from_utf8(text_bytes)?.chars() {
                if unic_emoji_char::is_emoji(c) {
                    s.push_str(&deunicode::deunicode_with_tofu(&c.to_string(), ""));
                } else {
                    s.push(c);
                }
            }
            s
        };
        
        Ok(text)
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            regex_items: Vec::new(),
            automaton: AhoCorasick::new(Vec::<String>::new()).unwrap(),
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

impl<'de> Deserialize<'de> for Dictionary {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let items: Vec<DictItem> = Vec::deserialize(deserializer)?;
        Ok(Dictionary::from_iter(items))
    }
}

impl Extend<DictItem> for Dictionary {
    fn extend<T: IntoIterator<Item = DictItem>>(&mut self, iter: T) {
        let (regex_items, items): (Vec<DictItem>, Vec<DictItem>) = iter.into_iter().partition(|item| item.is_regex);
        for item in regex_items.iter().chain(items.iter()) {
            self.keys.insert(item.key.clone());
        }
        self.items.extend(items);
        self.regex_items.extend(regex_items);
        self.automaton = AhoCorasick::new(self.items.iter().map(|item| item.key.clone())).unwrap();
    }
}

impl FromIterator<DictItem> for Dictionary {
    fn from_iter<T: IntoIterator<Item = DictItem>>(iter: T) -> Self {
        let mut dict = Dictionary::new();
        dict.extend(iter);
        dict
    }
}
