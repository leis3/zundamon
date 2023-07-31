use crate::config::{CONFIG_DIR, GLOBAL_CONFIG_FILE, GlobalConfig};
use std::path::Path;
use std::io::Write;
use anyhow::Result;
use serenity::model::prelude::UserId;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(subcommand)]
    pub cmd: Option<Command>
}

#[derive(Debug, StructOpt)]
pub enum Command {
    Config(ConfigOpt)
}

#[derive(Debug, StructOpt)]
pub struct ConfigOpt {
    /// Add ID of the user to have administrative privileges
    #[structopt(short, long)]
    pub add_admin: Option<UserId>,
    /// Remove ID of the user to have administrative privileges
    #[structopt(short, long)]
    pub remove_admin: Option<UserId>
}


pub fn config(opt: ConfigOpt) -> Result<()> {
    let mut global_config = GlobalConfig::load()?;

    if let Some(admin) = opt.add_admin {
        if !global_config.admin_user.contains(&admin) {
            global_config.admin_user.push(admin);
        }
    }

    if let Some(admin) = opt.remove_admin {
        if let Some(index) = global_config.admin_user.iter().position(|&user| user == admin) {
            global_config.admin_user.swap_remove(index);
        }
    }

    global_config.save()
}
