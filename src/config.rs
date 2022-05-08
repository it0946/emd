use anyhow::Context;
use regex::Regex;
use std::{
    fs,
    io::{BufReader, Read},
};

pub const CONFIG_FILE: &str = "mods.toml";

#[derive(serde::Deserialize)]
pub struct Config {
    #[serde(rename = "minecraft-version")]
    pub mc_version: Option<String>,
    #[serde(rename = "simoltaneous-downloads")]
    pub concurrency: Option<usize>,
    #[serde(rename = "mod-loader")]
    pub loader: Option<String>,
    pub destination: Option<String>,
    pub modrinth: Option<Vec<String>>,
    pub github: Option<Vec<String>>,
}

impl Config {
    pub fn read() -> anyhow::Result<Self> {
        let file = fs::File::open(CONFIG_FILE)
            .with_context(|| format!("failed to open `{}`", CONFIG_FILE))?;

        let mut br = BufReader::new(file);
        let mut res = String::new();

        br.read_to_string(&mut res)
            .with_context(|| format!("failed to read {}", CONFIG_FILE))?;

        let config = toml::from_str::<Self>(&res)?;

        let mr = config.modrinth.as_ref();
        let gr = config.github.as_ref();

        if config.mc_version.is_none() {
            bail!("You must provide a version of minecraft");
        } else if config.loader.is_none() {
            bail!("You must provide a mod loader");
        } else if mr.is_none() && gr.is_none() {
            bail!("You must provide at least one mod");
        }

        let glen = gr.unwrap_or(&vec![]).len();
        let mlen = mr.unwrap_or(&vec![]).len();

        if glen + mlen == 0 {
            bail!("You must provide at least one mod");
        }

        let check_version =
            Regex::new(r"^1.\d{1,2}(?:\.\d{1,2}|)$").expect("regex failed to compile");
        if !check_version.is_match(config.mc_version.as_ref().unwrap()) {
            bail!("Invalid minecraft version")
        }

        Ok(config)
    }
}
