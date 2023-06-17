use crate::dictionary::Dictionary;
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use serenity::model::prelude::GuildId;
use serde::{Serialize, Deserialize};
use anyhow::Result;

pub const CONFIG_DIR: &str = "config";
pub const CONFIG_FILE: &str = "config.json";
pub const DICT_FILE: &str = "dictionary.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config(HashMap<GuildId, GuildConfig>);

#[non_exhaustive]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GuildConfig {
    pub time_signal: bool,
    pub autojoin: bool,
    #[serde(skip)]
    pub dictionary: Dictionary
}

impl GuildConfig {
    pub fn load(guild_id: GuildId) -> Result<Self> {
        let path = Path::new(CONFIG_DIR).join(guild_id.0.to_string()).join(CONFIG_FILE);
        if let Ok(data) = std::fs::read_to_string(&path) {
            Ok(Self {
                dictionary: Dictionary::load(guild_id)?,
                ..serde_json::from_str(&data)?
            })
        } else {
            std::fs::create_dir_all(path.parent().unwrap())?;
            let mut file = std::fs::File::create(path)?;
            let config = Self::default();
            writeln!(file, "{}", serde_json::to_string_pretty(&config)?)?;
            Ok(config)
        }
    }

    pub fn save(&self, guild_id: GuildId) -> Result<()> {
        let path = Path::new(CONFIG_DIR).join(guild_id.0.to_string()).join(CONFIG_FILE);
        let mut file = std::fs::File::open(path)?;
        writeln!(file, "{}", serde_json::to_string_pretty(&self)?)?;
        self.dictionary.save()
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config = Self::default();
        for dir in std::fs::read_dir(CONFIG_DIR)? {
            let guild_id = GuildId(dir?.file_name().into_string().unwrap().parse()?);
            let guild_config = GuildConfig::load(guild_id)?;
            config.0.insert(guild_id, guild_config);
        }
        Ok(config)
    }

    pub fn reload(&mut self) -> Result<()> {
        *self = Self::load()?;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        for (&guild_id, config) in &self.0 {
            config.save(guild_id)?;
        }
        Ok(())
    }

    pub fn guild_config(&mut self, guild_id: GuildId) -> &GuildConfig {
        self.0.entry(guild_id).or_insert_with(|| {
            GuildConfig::load(guild_id).unwrap()
        });
        self.0.get(&guild_id).unwrap()
    }

    pub fn guild_config_mut(&mut self, guild_id: GuildId) -> &mut GuildConfig {
        self.0.entry(guild_id).or_insert_with(|| {
            GuildConfig::load(guild_id).unwrap()
        });
        self.0.get_mut(&guild_id).unwrap()
    }
}


impl Drop for Config {
    fn drop(&mut self) {
        let _ = self.save();
    }
}
