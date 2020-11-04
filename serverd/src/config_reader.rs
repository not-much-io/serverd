use internal_prelude::library_prelude::*;
use serde::Deserialize;
use std::{fs::File, path::Path};

const CONFIG_PATH_FALLBACK: &str = "/etc/serverd.conf";

#[derive(Deserialize, Debug)]
pub struct ServerdConfig {
    pub monitoring_service_config: Option<MonitoringServiceConfig>,
}

impl Default for ServerdConfig {
    fn default() -> Self {
        ServerdConfig {
            monitoring_service_config: None,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct MonitoringServiceConfig {
    pub monitors: Vec<Monitor>,
}

#[derive(Deserialize, Debug)]
pub struct Monitor {
    pub cmd_to_monitor: String,
    pub cmd_to_trigger: String,
    pub interval:       i32,
    pub delay:          i32,
}

pub fn read_config(config_path: Option<&Path>) -> Result<ServerdConfig> {
    if let Some(path) = config_path {
        if path.exists() {
            let file = File::open(path)?;
            serde_json::from_reader(file)?
        }
    }

    if Path::new(CONFIG_PATH_FALLBACK).exists() {
        let file = File::open(CONFIG_PATH_FALLBACK)?;
        serde_json::from_reader(file)?
    }

    Ok(ServerdConfig::default())
}
