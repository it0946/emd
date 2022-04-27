use super::{DownloadableMod, Mod};
use anyhow::Context;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct ModrinthFile {
    filename: String,
    url: String,
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

pub async fn get_from_modrinth(
    m: &Mod,
    client: &Client,
    version: &str,
    mod_loader: &str,
) -> anyhow::Result<DownloadableMod> {
    if m.mod_name.contains(" ") {
        // this is quite a basic check, need more for the future
        return Err(anyhow!("Invalid modname"));
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
                .filter(|s| s.contains(version))
                .count()
                != 0;
            if !version {
                return false;
            }

            res.loaders.iter().filter(|s| **s == mod_loader).count() != 0
        })
        .collect::<Vec<_>>();

    if remaining.len() == 0 {
        return Err(anyhow!(
            "Couldn't find a matching version of ({}) of {}",
            version,
            m.mod_name
        ));
    }

    let link = std::mem::take(&mut remaining[0].files[0].url);
    let filename = std::mem::take(&mut remaining[0].files[0].filename);

    Ok(DownloadableMod::new(filename, link))
}
