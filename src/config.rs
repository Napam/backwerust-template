use std::env;
use std::path::PathBuf;

use anyhow::{Context, bail};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub db_migrate_on_start: bool,
    pub db_path: PathBuf,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: non_empty_env("HOST").unwrap_or_else(|| "localhost".to_owned()),
            port: match non_empty_env("PORT") {
                Some(value) => value
                    .parse::<u16>()
                    .with_context(|| format!("PORT must be a port number, got {value:?}"))?,
                None => 6900,
            },
            db_migrate_on_start: env_bool("DB_MIGRATE_ON_START")?.unwrap_or(false),
            db_path: non_empty_env("DB_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("./data/sqlite.db")),
        })
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn non_empty_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

fn env_bool(name: &str) -> anyhow::Result<Option<bool>> {
    let Some(value) = non_empty_env(name) else {
        return Ok(None);
    };
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "t" | "yes" | "y" => Ok(Some(true)),
        "0" | "false" | "f" | "no" | "n" => Ok(Some(false)),
        _ => bail!("{name} must be a boolean, got {value:?}"),
    }
}
