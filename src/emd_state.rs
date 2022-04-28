use crate::{
    config::Config,
    sources::{Mod, ModSource},
};
use reqwest::Client;
use std::collections::HashSet;
use std::path::PathBuf;
use std::{ops::Range, sync::Arc};

pub struct EmdState {
    mod_loader: String,
    mc_version: String,
    mod_list: Vec<Mod>,
    concurrency: usize,
    destination: PathBuf,
}

impl EmdState {
    pub fn init() -> anyhow::Result<Self> {
        let config = Config::read()?;

        // Unwraps here are safe, because they are checked in Config::read
        let mod_loader = config.loader.unwrap();
        let mc_version = config.mc_version.unwrap();
        let concurrency = config.concurrency.unwrap_or(Self::determine_worker_count());

        let mod_list = {
            let modrinth = config
                .modrinth
                .unwrap_or(vec![])
                .into_iter()
                .map(|name| Mod::new(name, ModSource::Modrinth))
                .collect::<Vec<Mod>>();

            let github = config
                .github
                .unwrap_or(vec![])
                .into_iter()
                .map(|name| Mod::new(name, ModSource::Github))
                .collect::<Vec<Mod>>();

            [modrinth, github].concat()
        };

        let destination = PathBuf::from(config.destination.unwrap_or(".".into()));
        if !destination.exists() {
            return Err(anyhow!("Path specified by destination does not exist"));
        }

        let emd_state = Self {
            concurrency,
            mc_version,
            mod_list,
            mod_loader,
            destination,
        };

        Ok(emd_state)
    }

    pub async fn run(mut self) {
        println!("Downloading mods:");

        self.check_duplicates();
        let slice_indices = self.get_slice_indices();

        let emd_state = Arc::new(self);
        let client = Client::new();
        let mut join_handles = vec![];

        for range in slice_indices {
            let c_emd_state = emd_state.clone();
            let c_client = client.clone();

            join_handles.push(tokio::spawn(async move {
                Self::download_task(c_emd_state, c_client, range).await;
            }))
        }

        for handle in join_handles {
            if let Err(e) = handle.await {
                println!("Joining thread failed: {}", e);
            }
        }
    }

    async fn download_task(emd_state: Arc<Self>, client: Client, slice_range: Range<usize>) {
        let slice = &(*emd_state).mod_list[slice_range];
        let version = (*emd_state).mc_version.as_str();
        let mod_loader = (*emd_state).mod_loader.as_str();
        let path = &(*emd_state).destination;

        for m in slice {
            match m.get_url(&client, version, mod_loader).await {
                Ok(ok) => {
                    if path.join(&ok.filename).exists() {
                        println!("\t{} already exists", ok.filename);
                        continue;
                    }

                    if let Err(e) = ok.download_mod(&client, path).await {
                        println!("\tfailed to download {}: {}", m.mod_name, e);
                    } else {
                        println!("\tdownloaded {}", m.mod_name);
                    }
                }
                Err(e) => {
                    println!("\tfailed to download {}: {}", m.mod_name, e);
                }
            }
        }
    }

    fn check_duplicates(&mut self) {
        let mut unique_mods = HashSet::new();
        self.mod_list.retain(|m| {
            let name = if let Some((_, name)) = m.mod_name.split_once("/") {
                name.to_string()
            } else {
                m.mod_name.clone()
            }
            .to_lowercase();

            // This may not work for duplicates from github and modrinth, since the repo could be different
            let unique = unique_mods.insert(name);
            if !unique {
                println!("\tduplicate mod warning: {}", m.mod_name);
            }
            unique
        });
    }

    fn get_slice_indices(&self) -> Vec<Range<usize>> {
        let len = self.mod_list.len();

        if len < self.concurrency && len < 3 {
            return vec![0..len];
        }

        let mut slices = vec![];
        let mut c = 0;

        for nth in 0..self.concurrency {
            let n = c + {
                let mut n = 0;
                while nth + self.concurrency * n < len {
                    n += 1;
                }
                n
            };

            slices.push(c..n);
            c = n;
        }

        slices
    }

    fn determine_worker_count() -> usize {
        let cpu_count = num_cpus::get();

        match cpu_count {
            10.. => 6,
            6..=9 => 4,
            2..=5 => 2,
            _ => 1,
        }
    }
}
