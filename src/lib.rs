//! Library for maintaining configs on disk in a simple, ergonomic way.
//!
//! # Quickstart
//!
//! TODO
//!
//! For more examples, see the `examples/` directory.
//!
//! # Features
//!
//! - Configs are stored in JSON format.
//! - Config files are created with user-only permissions (0600) in case they contain sensitive
//!   data.

use std::{
    any,
    fmt::{self, Debug},
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
};

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error as ThisError;

mod environment;

/// Generic struct for managing an app's chunk of config data on disk.
///
/// Saves config files in $ILO_CONFIG_HOME, or ~/.config/ilo/ if the former is not set.
///
/// About the DeserializeOwned trait bound: see https://serde.rs/lifetimes.html.
/// Since the struct itself is loading the data from a file, it's in command of its own deserializer
/// lifetimes.
pub struct Config<TConfigData: Serialize + DeserializeOwned + Default> {
    config_data: TConfigData,
    config_file_key: String, // e.g. `jira` for ~/.config/ilo/jira.json
}

// If the config_data type is Debug, also implement Debug for the Config wrapper.
impl<TConfigData: Serialize + DeserializeOwned + Default + Debug> Debug for Config<TConfigData> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Config<{}> {{ config_data: {:?}, config_file_key: {} }}",
            any::type_name::<TConfigData>(),
            self.config_data,
            self.config_file_key,
        )
    }
}

impl<TConfigData: Serialize + DeserializeOwned + Default> Config<TConfigData> {
    /// Load a config based on a key.
    ///
    /// The file and directory creation is lazy, i.e. if the JSON file does not exist, a default
    /// config will be loaded and the file will not actually be created until there is a write.
    pub fn load(config_file_key: &str) -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path(config_file_key)?;

        let config_data = if config_path.is_file() {
            let file = File::open(&config_path)
                .map_err(|e| ConfigError::ConfigFileLoadError(config_path.clone(), e))?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader)
                .map_err(|e| ConfigError::ConfigFileParseError(config_path, e))?
        } else {
            TConfigData::default()
        };

        Ok(Self {
            config_data,
            config_file_key: config_file_key.to_string(),
        })
    }

    /// Flush config changes to disk.
    pub fn save(&self) -> Result<(), ConfigError> {
        // First check the directory
        let config_root = Self::get_config_root()?;
        match config_root.try_exists() {
            Ok(true) => (),
            Ok(false) => {
                fs::create_dir_all(config_root.clone())
                    .map_err(|e| ConfigError::ConfigRootCreateError(config_root, e))?;
            }
            Err(e) => {
                return Err(ConfigError::ConfigRootLoadError(config_root, e));
            }
        }

        let config_path = Self::get_config_path(&self.config_file_key)?;
        match config_path.try_exists() {
            Ok(exists) => {
                let mut options = OpenOptions::new();
                options.create(true).write(true).truncate(true);

                // If file needs to be created and we are on UNIX, set permissions to user-only
                #[cfg(unix)]
                {
                    if !exists {
                        options.mode(0o600);
                    }
                }

                match options.open(config_path.clone()) {
                    Ok(f) => {
                        let writer = BufWriter::new(f);
                        serde_json::to_writer_pretty(writer, &self.config_data)
                            .map_err(ConfigError::ConfigFileSerializeError)
                    }
                    Err(e) => Err(ConfigError::ConfigFileWriteError(config_path, e)),
                }
            }
            Err(e) => Err(ConfigError::ConfigFileWriteError(config_path, e)),
        }
    }

    #[inline]
    pub fn data(&self) -> &TConfigData {
        &self.config_data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut TConfigData {
        &mut self.config_data
    }

    fn get_config_root() -> Result<PathBuf, ConfigError> {
        let environment = environment::load_env();
        let config_root = environment
            .ilo_config_home
            .as_deref()
            .map(PathBuf::from)
            .or(home::home_dir().map(|d| d.join(".config").join("ilo")));

        match config_root {
            None => Err(ConfigError::NoHome),
            Some(root) => Ok(root),
        }
    }

    fn get_config_path(config_file_key: &str) -> Result<PathBuf, ConfigError> {
        Self::get_config_root().map(|root| root.join(format!("{}.json", config_file_key)))
    }
}

#[derive(ThisError, Debug)]
pub enum ConfigError {
    #[error("$ILO_CONFIG_HOME is not set and user home directory could not be determined")]
    NoHome,

    #[error("Config root dir {0} could not be loaded: {1}")]
    ConfigRootLoadError(PathBuf, io::Error),

    #[error("Config root dir does not exist at {0} and could not be created: {1}")]
    ConfigRootCreateError(PathBuf, io::Error),

    #[error("Config path exists at {0} but config could not be loaded: {1}")]
    ConfigFileLoadError(PathBuf, io::Error),

    #[error("Config path exists at {0} but JSON could not be parsed: {1}")]
    ConfigFileParseError(PathBuf, serde_json::Error),

    #[error("Config path location {0} could not be opened for writing: {1}")]
    ConfigFileWriteError(PathBuf, io::Error),

    #[error("There was an error serializing config to disk: {0}")]
    ConfigFileSerializeError(serde_json::Error),
}
