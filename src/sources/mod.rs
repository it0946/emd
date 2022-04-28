use std::path::PathBuf;

use anyhow::Context;
use reqwest::Client;
use tokio::{fs, io::AsyncWriteExt};

mod github;
mod modrinth;

pub struct DownloadableMod {
    pub filename: String,
    download_link: String,
}

impl DownloadableMod {
    pub fn new(filename: String, download_link: String) -> Self {
        Self {
            filename,
            download_link,
        }
    }

    pub async fn download_mod(self, client: &Client, destination: &PathBuf) -> anyhow::Result<()> {
        let response = client
            .get(self.download_link)
            .send()
            .await
            .with_context(|| "download request failed")?;

        let mut out_file = fs::File::create(destination.join(self.filename))
            .await
            .with_context(|| "failed to create out_file")?;

        let bytes = response
            .bytes()
            .await
            .expect("failed to convert response into bytes");

        out_file
            .write_all(&bytes)
            .await
            .with_context(|| "writing to file failed")?;

        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ModSource {
    Modrinth,
    Github,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Mod {
    pub mod_name: String,
    source: ModSource,
}

impl Mod {
    pub fn new(name: String, source: ModSource) -> Self {
        Self {
            mod_name: name,
            source,
        }
    }

    pub async fn get_url(
        &self,
        client: &Client,
        version: &str,
        mod_loader: &str,
    ) -> anyhow::Result<DownloadableMod> {
        match self.source {
            ModSource::Modrinth => modrinth::from_modrinth(self, client, version, mod_loader).await,
            ModSource::Github => github::from_github(self, client, version).await,
        }
    }
}
