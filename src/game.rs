use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::config::Config;

pub struct Game {
    proton_path: PathBuf,
    path: PathBuf,
    env: BTreeMap<String, String>,
}

impl Game {
    pub fn new(path: PathBuf, env: BTreeMap<String, String>, config: &Config) -> Self {
        let proton_path = config.proton.path.join("proton");

        Self { path, env, proton_path }
    }

    pub fn run(self) -> Result<()> {
        println!("{:?}", self.path);
        Command::new(&self.proton_path)
            .current_dir(self.path.parent().ok_or_else(|| anyhow!("Executable has no parent"))?)
            .arg("run")
            .arg(self.path)
            .envs(self.env)
            .spawn()
            .context("Couldn't start game")?;

        Ok(())
    }
}
