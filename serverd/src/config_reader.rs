use internal_prelude::library_prelude::*;
use serde::Deserialize;
use std::{fs::File, path::Path};

const CONFIG_PATH_FALLBACK: &str = "/etc/serverd.conf";

#[derive(Deserialize, Debug)]
pub struct ServerdConfig {}

impl Default for ServerdConfig {
    fn default() -> Self {
        ServerdConfig {}
    }
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
