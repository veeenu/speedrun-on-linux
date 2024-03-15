use std::{collections::BTreeMap, path::PathBuf, process::Command};

use anyhow::{anyhow, Context, Result};

use crate::config::Config;

pub struct Game {
    proton_path: PathBuf,
    path: PathBuf,
    env: BTreeMap<String, String>,
}

impl Game {
    pub fn new(path: PathBuf, env: BTreeMap<String, String>, config: &Config) -> Self {
        let proton_path = config.proton.path.join("dist/bin/wine64");

        Self { path, env, proton_path }
    }

    pub fn run(self) -> Result<()> {
        Command::new(&self.proton_path)
            .current_dir(self.path.parent().ok_or_else(|| anyhow!("Executable has no parent"))?)
            .arg("C:\\windows\\system32\\steam.exe")
            .arg(self.path)
            .envs(self.env)
            .spawn()
            .context("Couldn't start game")?;

        Ok(())
    }
}
