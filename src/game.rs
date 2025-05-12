use std::path::PathBuf;
use std::process::Command;
use std::{collections::BTreeMap, path::Path};

use anyhow::{anyhow, bail, Context, Result};

use crate::config::{Config, GameConfig};

pub struct Game {
    proton_path: PathBuf,
    argv: Vec<String>,
    env: BTreeMap<String, String>,
    gamescope: bool,
}

impl Game {
    pub fn new(
        argv: Vec<String>,
        env: BTreeMap<String, String>,
        config: &Config,
        game_config: &GameConfig,
    ) -> Self {
        let proton_path = config.proton.path.join("proton");
        let gamescope = game_config.gamescope;

        Self { argv, env, proton_path, gamescope }
    }

    pub fn run(self) -> Result<()> {
        println!("Running game {:?}", self.argv);
        let Some(path) = self.argv.first() else {
            bail!("Game has empty command line!");
        };

        let cwd = Path::new(path).parent().ok_or_else(|| anyhow!("Executable has no parent"))?;

        if self.gamescope {
            Command::new("gamescope")
                .current_dir(cwd)
                .args([
                    "gamescope",
                    "-w",
                    "2048",
                    "-h",
                    "1152",
                    "-b",
                    "-r",
                    "60",
                    "--mangoapp",
                    // "--fullscreen",
                    "--",
                ])
                .arg("run")
                .args(self.argv)
                .envs(self.env)
                .spawn()
                .context("Couldn't start game")?;
        } else {
            Command::new(&self.proton_path)
                .current_dir(cwd)
                .arg("run")
                .args(self.argv)
                .envs(self.env)
                .spawn()
                .context("Couldn't start game")?;
        }

        Ok(())
    }
}
