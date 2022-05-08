use super::{DownloadableMod, Mod};
use anyhow::Context;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct ModrinthFile {
    filename: String,
    url: String,
    primary: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ModrinthResponse {
    name: String,
    files: Vec<ModrinthFile>,
    // dependencies: Option<Vec<>>, // these may be useful, but not doing this for now
    game_versions: Vec<String>,
    loaders: Vec<String>,
}

pub async fn from_modrinth(
    m: &Mod,
    client: &Client,
    // version: &str,
    version: &Regex,
    mod_loader: &str,
) -> anyhow::Result<DownloadableMod> {
    // TODO: this kind of check should probably move to EmdState::run after checking for duplicates
    if m.mod_name.contains(" ") {
        // this is quite a basic check, need more for the future
        // return Err(anyhow!("Invalid modname"));
        bail!("Invalid mod name");
    }

    let url = format!("https://api.modrinth.com/v2/project/{}/version", m.mod_name);
    let res = client
        .get(url)
        .send()
        .await
        .with_context(|| "Failed to send request")?
        .error_for_status()
        .with_context(|| "Request returned an error")?;

    let res_list = res
        .json::<Vec<ModrinthResponse>>()
        .await
        .expect("failed to parse version list");

    let mut remaining = res_list
        .into_iter()
        .filter(|res| {
            let version = res
                .game_versions
                .iter()
                // .filter(|s| s.contains(version))
                .filter(|s| version.is_match(s))
                .count()
                != 0;
            if !version {
                return false;
            }

            res.loaders.iter().filter(|s| **s == mod_loader).count() != 0
        })
        .collect::<Vec<_>>();

    let mut preferred = 0;

    if remaining.len() == 0 {
        bail!("Couldn't find a matching version of {}", m.mod_name);
    } else if remaining.len() > 1 {
        for (i, file) in remaining[0].files.iter().enumerate() {
            if file.primary {
                preferred = i;
            }
        }
    }

    let link = std::mem::take(&mut remaining[0].files[preferred].url);
    let filename = std::mem::take(&mut remaining[0].files[preferred].filename);

    Ok(DownloadableMod::new(filename, link))
}
