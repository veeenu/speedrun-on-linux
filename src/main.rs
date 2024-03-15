use speedrun_on_linux::{config::load_config, livesplit::LiveSplit};

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config()?;

    LiveSplit::new(&config)
        .context("Couldn't install LiveSplit")?
        .run()
        .context("Couldn't run LiveSplit")?;

    Ok(())
}
