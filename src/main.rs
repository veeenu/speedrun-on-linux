use speedrun_on_linux::{config::load_config, livesplit::LiveSplit};

use anyhow::{anyhow, Context, Result};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};

mod flags {
    xflags::xflags! {
        cmd app {
            /// Run LiveSplit.
            cmd livesplit { }
            /// Run game
            cmd game {
                /// Specify game name.
                required name: String
                /// Specify game version.
                optional version: String
            }
        }
    }
}

fn select_version<'a>(choices: &'a [&'a String]) -> Result<String> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .default(0)
        .items(choices)
        .interact_on_opt(&Term::stderr());

    match selection {
        Ok(Some(idx)) => Ok(choices[idx].to_string()),
        Ok(None) => Err(anyhow!("Nothing was selected")),
        Err(e) => Err(anyhow!(e)),
    }
}

fn main() -> Result<()> {
    let config = load_config()?;

    let flags = flags::App::from_env_or_exit();

    match flags.subcommand {
        flags::AppCmd::Livesplit(_) => {
            LiveSplit::new(&config)
                .context("Couldn't install LiveSplit")?
                .run()
                .context("Couldn't run LiveSplit")?;
        },
        flags::AppCmd::Game(g) => {
            let version = match g.version {
                Some(v) => v.to_string(),
                None => config
                    .games
                    .versions(&g.name)
                    .ok_or_else(|| anyhow!("Couldn't find game: {}", g.name))
                    .and_then(|versions| select_version(&versions))?,
            };
            let game =
                config.games.get(&g.name, &version, &config).context("Couldn't setup game")?;
            game.run().context("Couldn't run game")?;
        },
    }

    Ok(())
}
