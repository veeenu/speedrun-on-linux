use std::env;
use std::io::Cursor;
use std::process::Command;

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::config::Config;
use crate::util::{self, find_running_steam_appid};

// TODO make dynamic
const LIVESPLIT_DOWNLOAD: &str =
    "https://github.com/LiveSplit/LiveSplit/releases/download/1.8.27/LiveSplit_1.8.27.zip";

// TODO make dynamic
const SOULSPLITTER_DOWNLOAD: &str =
    "https://github.com/FrankvdStam/SoulSplitter/releases/download/1.5.3/1.5.3.zip";

pub struct LiveSplit {
    config: Config,
}

impl LiveSplit {
    pub fn new(config: &Config) -> Result<Self> {
        if !config.livesplit.path.join("LiveSplit.exe").exists() {
            download_livesplit(config).context("Couldn't download LiveSplit")?;
        }

        if !config.livesplit.path.join("Components/SoulSplitter.dll").exists() {
            download_soulsplitter(config).context("Couldn't download SoulSplitter")?;
        }

        Ok(LiveSplit { config: config.clone() })
    }

    pub fn run(&self) -> Result<()> {
        let steam_appid = find_running_steam_appid()?.to_string();

        Command::new(self.config.proton.path.join("proton"))
            .arg("run")
            .arg(self.config.livesplit.path.join("LiveSplit.exe"))
            .env(
                "STEAM_COMPAT_DATA_PATH",
                self.config.steam.path.join("steamapps/compatdata").join(&steam_appid),
            )
            .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &self.config.steam.path)
            .env(
                "PATH",
                format!("{:?}:{}", self.config.proton.path, env::var("PATH").unwrap_or_default()),
            )
            .env("APPID", &steam_appid)
            .spawn()
            .context("Couldn't run LiveSplit")?;

        Ok(())
    }
}

fn download_livesplit(config: &Config) -> Result<()> {
    let bytes = util::download_file(LIVESPLIT_DOWNLOAD, "Downloading LiveSplit...")?;
    ZipArchive::new(Cursor::new(bytes))
        .context("Couldn't open archive")?
        .extract(&config.livesplit.path)
        .context("Couldn't extract archive")?;

    Ok(())
}

fn download_soulsplitter(config: &Config) -> Result<()> {
    let bytes = util::download_file(SOULSPLITTER_DOWNLOAD, "Downloading SoulSplitter...")?;
    ZipArchive::new(Cursor::new(bytes))
        .context("Couldn't open archive")?
        .extract(&config.livesplit.path.join("Components"))
        .context("Couldn't extract archive")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TestConfig;

    #[test]
    fn test_livesplit() {
        let config = TestConfig::new();

        LiveSplit::new(&config.config).expect("Couldn't construct livesplit");
    }
}
