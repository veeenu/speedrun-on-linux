use std::io::Read;

use anyhow::{anyhow, bail, Result};
use indicatif::{ProgressBar, ProgressStyle};
use sysinfo::System;

pub fn download_file(url: &str, msg: &'static str) -> Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();

    let mut response = reqwest::blocking::get(url)?;

    let total_size = response.content_length().unwrap_or(0);
    let progress = ProgressBar::new(total_size);
    progress.set_style(
        ProgressStyle::with_template(
            "{msg}\n[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} \
             ({bytes_per_sec}, {eta})",
        )
        .unwrap(),
    );
    progress.set_message(msg);

    let mut downloaded = 0;
    let mut chunk = [0; 4096];
    while let Ok(size) = response.read(&mut chunk) {
        if size == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..size]);
        downloaded += size as u64;
        progress.inc(size as u64);
    }

    progress.finish();

    if downloaded < total_size {
        bail!("Download incomplete!");
    }

    Ok(buf)
}

pub fn find_running_steam_appid() -> Result<u64> {
    let sys = System::new_all();

    let Some((_, process)) = sys.processes().iter().find(|(_, process)| {
        process.cmd().first().map(|s| s.to_lowercase().ends_with("steam.exe")).unwrap_or(false)
    }) else {
        bail!("Couldn't find a steam process running. Start a game first.");
    };

    process
        .environ()
        .iter()
        .find_map(|s| {
            let mut it = s.split('=');
            it.next().filter(|&s| s == "SteamAppId")?;
            it.next()?.parse().ok()
        })
        .ok_or_else(|| anyhow!("Could not find steam appid in steam.exe's environment"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_running_steam_appid() {
        println!("{:?}", find_running_steam_appid());
    }
}
