# ilo-config

Library for maintaining configs on disk in a simple, ergonomic way.

## Quickstart

1.  Create a new Cargo binary package.

    ```sh
    cargo new ilo-config-example && cd ilo-config-example
    ```

1.  Install the dependencies.  `serde` and `reqwest` are only needed for the example code; they're
    not necessary for using the library in general.

    ```sh
    cargo add ilo-config

    cargo add serde -F derive
    cargo add reqwest -F blocking
    ```

1.  Paste the example code into `src/main.rs`.

    ```rs
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
    ```

1.  Run `export ILO_CONFIG_HOME=$(pwd)` so that config files are created in the current directory
    instead of in the default directory (which is `~/.config/ilo`).
1.  Run the example with `cargo run`.
1.  (Optional) Inspect the generated `example-config.json` file.  Try changing the URL to, say,
    `https://httpbin.org/headers` and rerunning the program, or replacing the URL with a number and
    verifying that the type mismatch causes an error at runtime.

For more examples, see the [`examples/`](./examples) directory.
