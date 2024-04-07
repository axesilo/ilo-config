//! Bootstrap environment configuration for managing the rest of the configs.
//!
//! Users of ilo-config may set the root environment variable `ILO_CONFIG_HOME` to customize where
//! the rest of their configs are stored. If not set, this variable defaults to `~/.config/ilo/`.
use serde::Deserialize;

/// Env vars as a typed struct - for loading using the `envy` crate.
#[derive(Deserialize, Debug)]
pub struct IloConfigEnvironment {
    pub ilo_config_home: Option<String>,
}

/// Load the environment from environment variables.
///
/// # Panics
///
/// This function panics if the environment variables can't be loaded properly, since the rest of
/// the system can't do much without those.
pub fn load_env() -> IloConfigEnvironment {
    envy::from_env().expect("Failed to load configuration from environment variables")
}
