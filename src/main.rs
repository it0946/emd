mod config;
mod emd_state;
mod sources;

#[macro_use]
extern crate anyhow;

#[tokio::main]
async fn main() {
    match emd_state::EmdState::init() {
        Ok(state) => state.run().await,
        Err(e) => {
            println!("Failed to run: {}", e);
            std::process::exit(1);
        }
    }
}
