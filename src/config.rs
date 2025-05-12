use std::collections::BTreeMap;
use std::env::{self, VarError};
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context};
use serde::{de, Deserialize, Deserializer};
#[cfg(test)]
use tempfile::TempDir;

use crate::game::Game;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub steam: Steam,
    pub proton: Proton,
    pub livesplit: LiveSplit,
    #[serde(default)]
    pub games: GameConfigs,
    #[serde(default)]
    pub envs: GameEnvironments,
}

impl Config {
    pub fn versions(&self, name: &str) -> Option<Vec<&String>> {
        self.games.0.get(name).map(|g| g.paths.keys().collect())
    }

    pub fn get_game(&self, name: &str, version: &str, config: &Config) -> anyhow::Result<Game> {
        let mut env = BTreeMap::new();

        let Some(game) = self.games.0.get(name) else {
            bail!("Couldn't find game: {name}");
        };

        let env_name = game.env.as_deref().unwrap_or(name);

        self.envs.traverse(&mut env, env_name, config)?;

        let argv = match game.paths.get(version) {
            Some(GamePath::One(p)) => vec![p.clone()],
            Some(GamePath::Many(v)) if !v.is_empty() => v.clone(),
            Some(GamePath::Many(_)) | None => {
                bail!("Couldn't find path to version {version} for game {name}")
            },
        };

        let _ = env.entry("STEAM_COMPAT_DATA_PATH".to_string()).or_insert_with(|| {
            config
                .steam
                .path
                .join("steamapps/compatdata")
                .join(&game.app_id)
                .to_string_lossy()
                .to_string()
        });

        let _ = env
            .entry("STEAM_COMPAT_CLIENT_INSTALL_PATH".to_string())
            .or_insert_with(|| config.proton.path.to_string_lossy().to_string());

        Ok(Game::new(argv, env, config, game))
    }
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

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GamePath {
    One(String),
    Many(Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
pub struct GameConfig {
    pub app_id: String,
    pub env: Option<String>,
    #[serde(default)]
    pub gamescope: bool,
    #[serde(default)]
    pub paths: BTreeMap<String, GamePath>,
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct GameConfigs(pub BTreeMap<String, GameConfig>);

#[derive(Default, Deserialize, Debug, Clone)]
pub struct GameEnvironments(BTreeMap<String, GameEnvironment>);

impl GameEnvironments {
    fn traverse(
        &self,
        env: &mut BTreeMap<String, String>,
        current: &str,
        config: &Config,
    ) -> anyhow::Result<()> {
        let current =
            self.0.get(current).ok_or_else(|| anyhow!("{current}: environment not found"))?;

        for node in &current.deps.before {
            self.traverse(env, node, config)?;
        }

        current.apply(env, config)?;

        for node in &current.deps.after {
            self.traverse(env, node, config)?;
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct GameEnvironment {
    #[serde(default)]
    pub deps: EnvironmentDependencies,
    #[serde(flatten)]
    pub env: BTreeMap<String, String>,
}

impl GameEnvironment {
    fn apply(&self, env: &mut BTreeMap<String, String>, config: &Config) -> anyhow::Result<()> {
        for (k, v) in &self.env {
            let new_value =
                shellexpand::full_with_context(v, get_home_dir, |key| -> Result<_, VarError> {
                    Ok(match key {
                        "STEAM_PATH" => config.steam.path.to_str(),
                        "PROTON_PATH" => config.proton.path.to_str(),
                        key => env.get(key).map(|s| s.as_str()),
                    })
                })?;

            env.insert(k.to_string(), new_value.to_string());
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct EnvironmentDependencies {
    #[serde(default)]
    pub before: Vec<String>,
    #[serde(default)]
    pub after: Vec<String>,
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

    impl de::Visitor<'_> for ShellExpandVisitor {
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

pub fn load_config(path: Option<PathBuf>) -> anyhow::Result<Config> {
    let config_path = match path.or_else(config_path) {
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
            games: Default::default(),
            envs: Default::default(),
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
        let cfg = load_config(Some("speedrun-on-linux.toml".into())).expect("Couldn't load config");

        cfg.get_game("DarkSoulsIII", "1.08", &cfg).expect("No env");
    }
}
