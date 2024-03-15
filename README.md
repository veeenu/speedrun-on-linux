# speedrun-on-linux

<small>Also known as: rewriting shell scripts in Rust because surely that's a good idea.</small>

This is a configurable suite of shortcuts that I use for setting up and running my speedrun 
related games and tools.

## Installing

```sh
cargo install --git https://github.com/veeenu/speedrun-on-linux
```

Binary GitHub releases coming soon™.

## Configuration

At the first start, a configuration file is placed at `~/.config/speedrun-on-linux.toml`.

You **MUST** configure your games appropriately. 

### Environment variables

You need to figure out what environment variables Proton sets for your game. To do that,
you __TODO: tutorial about `PROTON_DUMP_DEBUG_COMMANDS`__.

There is bound to be a lot of overlap between games, so game environment variable sets can depend on
each other. You can specify these dependencies via the `games.'game'.env-deps` property; a set
can be evaluated `before` or `after` the game's own dependencies, and once a set is evaluated,
the corresponding variables will be available to sets evaluated afterwards.

A set of common variables is defined in the default configuration, under the `games.common` key.
Dark Souls III and Elden Ring both depend on that set, because those variables are fully identical 
between the two, but your mileage may vary. Feel free to open a PR if you find worthwhile 
adjustments that are truly universal.

#### Example

```toml
# ${PROTON_PATH} and ${STEAM_PATH} are already available, and set to the values
# of `steam.path` and `proton.path` that are at the top of the configuration file.
[games.common.env]
WINEDLLPATH="${PROTON_PATH}/dist/lib64/wine:${PROTON_PATH}/dist/lib/wine" 
WINEESYNC="1" 
WINEFSYNC="1" 
WINEPREFIX="${STEAM_PATH}/steamapps/compatdata/${SteamAppId}/pfx/" 
WINE_GST_REGISTRY_DIR="${STEAM_PATH}/steamapps/compatdata/${SteamAppId}/gstreamer-1.0/" 
WINE_LARGE_ADDRESS_AWARE="1" 

# These two variables are specific to Dark Souls III.
[games."DarkSoulsIII".env]
SteamAppId="374320" 
SteamGameId="374320" 

# `games.common.env` gets evaluated _after_ `games."DarkSoulsIII".env`,
# because it relies on the ${SteamAppId} provided by Dark Souls III.
[games."DarkSoulsIII".env-deps]
after = ["common"]
```

### Game location

You have to specify where the game is located on your system. Multiple patches are supported
natively. You can give whatever name you want to the patch, and the configuration is a key-value
mapping of version name to version path.

#### Example

```toml
# These paths should be changed manually.
[games."DarkSoulsIII".paths]
"1.04" = "/mnt/Patches/DARK SOULS III/1.04/Game/DarkSoulsIII.exe"
"1.08" = "/mnt/Patches/DARK SOULS III/1.08/Game/DarkSoulsIII.exe"
"1.12" = "/mnt/Patches/DARK SOULS III/1.12/Game/DarkSoulsIII.exe"
"1.15.1" = "/mnt/Patches/DARK SOULS III/1.15.1/Game/DarkSoulsIII.exe"
```

## Usage

### Run a game

To run a game you configured, use the `game` subcommand. You can either provide a version
name, or select it interactively.

```
# Select version interactively.
$ speedrun-on-linux game DarkSoulsIII
❯ 1.04
  1.08
  1.12
  1.15.1

# Run a specific version.
$ speedrun-on-linux game DarkSoulsIII 1.08
```

### Run LiveSplit

To start LiveSplit, first start the game you want to run: it is necessary for it to find out
the correct Wine prefix to run in. Then, with the game running:

```
speedrun-on-linux livesplit
```

If LiveSplit is not installed already, it will be downloaded and installed in the configured 
`livesplit.path` (`~/.local/share/LiveSplit` by default) alongside SoulSplitter, and then started.
