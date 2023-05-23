use crate::dictionary::Dictionary;
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use serenity::model::prelude::GuildId;
use serde::{Serialize, Deserialize};

pub const CONFIG_DIR: &str = "config";
pub const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config(pub HashMap<GuildId, GuildConfig>);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GuildConfig {
    pub dictionary: Dictionary,
    pub time_signal: bool
}

impl Config {
    pub fn load(&mut self, guild_id: GuildId) {
        let config_file = Path::new(CONFIG_DIR)
            .join(guild_id.0.to_string())
            .join(CONFIG_FILE);

        if let Ok(data) = std::fs::read_to_string(&config_file) {
            let config = serde_json::from_str(&data).unwrap();
            self.0.insert(guild_id, config);
        } else {
            std::fs::create_dir_all(config_file.parent().unwrap()).unwrap();
            std::fs::File::create(config_file).unwrap();
            self.0.insert(guild_id, GuildConfig::default());
        }
    }

    pub fn save(&self) {
        for (guild_id, config) in &self.0 {
            let config_file = Path::new(CONFIG_DIR)
                .join(guild_id.0.to_string())
                .join(CONFIG_FILE);
            let mut file = std::fs::File::create(config_file).unwrap();
            write!(file, "{}", serde_json::to_string_pretty(config).unwrap()).unwrap();
            file.flush().unwrap();
        }
    }
}


impl Drop for Config {
    fn drop(&mut self) {
        self.save();
    }
}
