use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use speedrun_on_linux::config::load_config;
use speedrun_on_linux::livesplit::LiveSplit;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run LiveSplit.
    Livesplit,
    /// Run game.
    Game { name: String, version: Option<String> },
    /// List games.
    List,
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

    let command = Args::parse();

    match command.command {
        Commands::Livesplit => {
            LiveSplit::new(&config)
                .context("Couldn't install LiveSplit")?
                .run()
                .context("Couldn't run LiveSplit")?;
        },
        Commands::Game { name, version } => {
            let version = match version {
                Some(v) => v.to_string(),
                None => config
                    .versions(&name)
                    .ok_or_else(|| anyhow!("Couldn't find game: {}", name))
                    .and_then(|versions| select_version(&versions))?,
            };
            let game = config.get_game(&name, &version, &config).context("Couldn't setup game")?;
            game.run().context("Couldn't run game")?;
        },
        Commands::List => {
            config.games.0.keys().for_each(|g| println!("{}", g));
        },
    }

    Ok(())
}
