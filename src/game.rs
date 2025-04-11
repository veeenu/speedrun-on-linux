use std::path::PathBuf;
use std::process::Command;
use std::{collections::BTreeMap, path::Path};

use anyhow::{anyhow, bail, Context, Result};

use crate::config::Config;

pub struct Game {
    proton_path: PathBuf,
    argv: Vec<String>,
    env: BTreeMap<String, String>,
}

impl Game {
    pub fn new(argv: Vec<String>, env: BTreeMap<String, String>, config: &Config) -> Self {
        let proton_path = config.proton.path.join("proton");

        Self { argv, env, proton_path }
    }

    pub fn run(self) -> Result<()> {
        println!("Running game {:?}", self.argv);
        let Some(path) = self.argv.get(0) else {
            bail!("Game has empty command line!");
        };

        let cwd = Path::new(path).parent().ok_or_else(|| anyhow!("Executable has no parent"))?;

        Command::new(&self.proton_path)
            .current_dir(cwd)
            .arg("run")
            .args(self.argv)
            .envs(self.env)
            .spawn()
            .context("Couldn't start game")?;

        Ok(())
    }
}
