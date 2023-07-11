use crate::config::Config;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serenity::{
    prelude::*,
    model::id::{GuildId, ChannelId}
};

pub struct TextChannelId;

impl TypeMapKey for TextChannelId {
    type Value = Arc<Mutex<HashMap<GuildId, ChannelId>>>;
}

pub struct ConfigData;

impl TypeMapKey for ConfigData {
    type Value = Arc<Mutex<Config>>;
}

pub struct ConnectedChannel;

impl TypeMapKey for ConnectedChannel {
    type Value = Arc<Mutex<HashMap<GuildId, ChannelId>>>;
}
