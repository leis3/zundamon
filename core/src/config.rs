use dictionary::Dictionary;
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use serenity::model::prelude::{GuildId, UserId};
use serde::{Serialize, Deserialize};
use anyhow::Result;

pub const CONFIG_DIR: &str = "config";
pub const CONFIG_FILE: &str = "config.json";
pub const DICT_FILE: &str = "dictionary.json";
pub const GLOBAL_CONFIG_FILE: &str = "global_config.json";

// デフォルトはノーマルずんだもん
fn default_speaker() -> u32 { 3 }

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub admin_user: Vec<UserId>
}

impl GlobalConfig {
    pub fn load() -> Result<Self> {
        let dir = Path::new(CONFIG_DIR);
        let config_path = dir.join(GLOBAL_CONFIG_FILE);
        if !config_path.exists() {
            std::fs::create_dir_all(dir)?;
            let mut file = std::fs::File::create(&config_path)?;
            let config = GlobalConfig::default();
            writeln!(file, "{}", serde_json::to_string_pretty(&config)?)?;
        }
        let config = std::fs::read_to_string(&config_path)?;
        Ok(serde_json::from_str(&config)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = Path::new(CONFIG_DIR).join(GLOBAL_CONFIG_FILE);
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "{}", serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config(HashMap<GuildId, GuildConfig>);

#[non_exhaustive]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GuildConfig {
    pub time_signal: bool,
    pub autojoin: bool,
    #[serde(default = "default_speaker")]
    pub speaker_id: u32,
    #[serde(skip)]
    pub dictionary: Dictionary
}

impl GuildConfig {
    pub fn load(guild_id: GuildId) -> Result<Self> {
        let dir = Path::new(CONFIG_DIR).join(guild_id.0.to_string());
        let config_path = dir.join(CONFIG_FILE);
        let dict_path = dir.join(DICT_FILE);
        if !config_path.exists() {
            std::fs::create_dir_all(&dir)?;
            let mut file = std::fs::File::create(&config_path)?;
            let config = Self::default();
            writeln!(file, "{}", serde_json::to_string_pretty(&config)?)?;
        }
        if !dict_path.exists() {
            std::fs::create_dir_all(&dir)?;
            let mut file = std::fs::File::create(&dict_path)?;
            let dict = Dictionary::new();
            writeln!(file, "{}", serde_json::to_string_pretty(&dict)?)?;
        }
        let config = std::fs::read_to_string(&config_path)?;
        let dict = std::fs::read_to_string(&dict_path)?;
        Ok(Self {
            dictionary: serde_json::from_str(&dict)?,
            ..serde_json::from_str(&config)?
        })
    }

    pub fn save(&self, guild_id: GuildId) -> Result<()> {
        let dir = Path::new(CONFIG_DIR).join(guild_id.0.to_string());
        let mut file = std::fs::File::create(dir.join(CONFIG_FILE))?;
        writeln!(file, "{}", serde_json::to_string_pretty(&self)?)?;
        let mut file = std::fs::File::create(dir.join(DICT_FILE))?;
        writeln!(file, "{}", serde_json::to_string_pretty(&self.dictionary)?)?;
        Ok(())
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
