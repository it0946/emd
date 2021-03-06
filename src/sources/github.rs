use crate::sources::{DownloadableMod, Mod};
use anyhow::Context;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct GithubAsset {
    name: String,
    // it might be a good idea to check if the filetype is jar
    // content_type: String,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct GithubResponse {
    tag_name: String,
    name: String,
    // body: String, probably shouldn't check this for the version
    assets: Vec<GithubAsset>,
}

pub async fn from_github(
    m: &Mod,
    client: &Client,
    // version: &str,
    version: &Regex,
    // mod_loader: &str, // I'm not sure where to check this in the response
) -> anyhow::Result<DownloadableMod> {
    let url = format!("https://api.github.com/repos/{}/releases", m.mod_name);
    let res = client
        .get(url)
        // All github requests require the User-Agent header
        .header("User-Agent", "eon-mod-downloader")
        .send()
        .await
        .with_context(|| "Failed to send request")?
        .error_for_status()
        .with_context(|| "Request returned an error")?;

    let res_list = res
        .json::<Vec<GithubResponse>>()
        .await
        .with_context(|| "Failed to parse response")?;

    let mut remaining = res_list
        .into_iter()
        .filter(|res| check_version(res, version))
        .collect::<Vec<GithubResponse>>();

    if remaining.len() == 0 {
        bail!(
            "Couldn't find a matching version of {} (this can fail on github)",
            m.mod_name
        );
    }

    let link = std::mem::take(&mut remaining[0].assets[0].browser_download_url);
    let filename = std::mem::take(&mut remaining[0].assets[0].name);

    Ok(DownloadableMod::new(filename, link))
}

fn check_version(m: &GithubResponse, version: &Regex) -> bool {
    if version.is_match(&m.tag_name) {
        true
    } else if version.is_match(&m.name) {
        true
    } else {
        false
    }
}
