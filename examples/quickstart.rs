//! Basic example of using ilo-config to load and save strongly-typed config data.
//!
//! Usage:
//!
//! ```sh
//! export ILO_CONFIG_HOME=$(pwd)  # Defaults to ~/.config/ilo/ if not specified
//! cargo run --example quickstart
//!
//! # Optional: edit the config file with a new URL and verify the change gets picked up
//! # (if not on macOS, remove the first `''` after `-i`)
//! sed -i '' 's#httpbin\.org/get#httpbin\.org/headers#g' example-config.json
//! cargo run --example quickstart
//!
//! # Optional: clean up config file that was created
//! rm example-config.json
//! ```
use ilo_config::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
struct QuickstartConfig {
    /// URL to which a GET request should be made
    url: Option<String>,

    /// Additional text to be saved in the config, e.g. reminder of how the file got created
    comment: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config: Config<QuickstartConfig> = Config::load("example-config")?;
    let data = config.data_mut();
    if data.comment.is_none() {
        data.comment = Some(String::from(
            "Created by the ilo-config package's quickstart example.",
        ));
    }

    let url: &str = data
        .url
        .get_or_insert_with(|| String::from("https://httpbin.org/get"));
    let response = reqwest::blocking::get(url)?;
    let text = response.text()?;

    println!("Response from configured URL: {text}");

    config.save().map_err(|e| e.into())
}
