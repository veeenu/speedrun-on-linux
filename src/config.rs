use std::{
    env::{self, VarError},
    fs,
    path::PathBuf,
};

use anyhow::Context;
use serde::{de, Deserialize, Deserializer};
#[cfg(test)]
use tempfile::TempDir;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub steam: Steam,
    pub proton: Proton,
    pub livesplit: LiveSplit,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Steam {
    #[serde(deserialize_with = "shell_expand")]
    pub path: PathBuf,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Proton {
    #[serde(deserialize_with = "shell_expand")]
    pub path: PathBuf,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LiveSplit {
    #[serde(deserialize_with = "shell_expand")]
    pub path: PathBuf,
}

fn get_home_dir() -> Option<String> {
    dirs::home_dir().map(|p| p.to_string_lossy().to_string())
}

fn get_env(name: &str) -> Result<Option<String>, VarError> {
    if let Ok(v) = env::var(name) {
        return Ok(Some(v));
    }

    let path = match name {
        "XDG_DATA_HOME" => dirs::data_dir().ok_or(VarError::NotPresent),
        "XDG_CONFIG_HOME" => dirs::config_dir().ok_or(VarError::NotPresent),
        "XDG_STATE_HOME" => dirs::state_dir().ok_or(VarError::NotPresent),
        "XDG_CACHE_HOME" => dirs::cache_dir().ok_or(VarError::NotPresent),
        "XDG_RUNTIME_HOME" => dirs::runtime_dir().ok_or(VarError::NotPresent),
        _ => Err(VarError::NotPresent),
    };

    path.map(|p| Some(p.to_string_lossy().to_string()))
}

fn shell_expand<'de, D: Deserializer<'de>>(deserializer: D) -> Result<PathBuf, D::Error> {
    struct ShellExpandVisitor;

    impl<'de> de::Visitor<'de> for ShellExpandVisitor {
        type Value = PathBuf;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a path (environment variables allowed)")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            shellexpand::full_with_context(&v, get_home_dir, get_env)
                .map(|s| PathBuf::from(s.as_ref()))
                .map_err(E::custom)
        }
    }

    deserializer.deserialize_any(ShellExpandVisitor)
}

fn config_path() -> Option<PathBuf> {
    [
        Some(PathBuf::from("speedrun-on-linux.toml")),
        dirs::config_dir().map(|p| p.join("speedrun-on-linux.toml")),
    ]
    .into_iter()
    .flatten()
    .find(|p| p.exists())
}

pub fn load_config() -> anyhow::Result<Config> {
    let config_path = match config_path() {
        Some(config_path) => config_path,
        None => {
            let config_path = dirs::config_dir()
                .map(|p| p.join("speedrun-on-linux.toml"))
                .context("Couldn't find configuration directory")?;
            fs::write(&config_path, include_str!("../speedrun-on-linux.toml"))
                .context("Couldn't write default configuration")?;
            config_path
        },
    };

    Ok(toml::from_str(&fs::read_to_string(config_path)?)?)
}

#[cfg(test)]
pub struct TestConfig {
    pub config: Config,
    pub temp_dir: TempDir,
}

#[cfg(test)]
impl TestConfig {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Couldn't create temp dir");
        eprintln!("Test config built at {:?}", temp_dir.path());
        let config = Config {
            steam: Steam { path: temp_dir.path().join("Steam") },
            proton: Proton { path: temp_dir.path().join("Steam/steamapps/common/Proton 8.0") },
            livesplit: LiveSplit { path: temp_dir.path().join("LiveSplit") },
        };

        Self { config, temp_dir }
    }
}

#[cfg(test)]
impl Default for TestConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        println!("{:#?}", load_config())
    }
}
