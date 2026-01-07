//! Reads configs.

use std::path::PathBuf;

use serde::Deserialize;

use crate::env::CONFIG_DIR;

/// The available config files.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigFile {
    /// The services to update.
    Services,
}

impl ConfigFile {
    /// The file name of the config file.
    pub fn file_name(&self) -> &'static str {
        match self {
            Self::Services => "services.toml",
        }
    }

    /// The path to the config file, respecting [`CONFIG_DIR`].
    pub fn path(&self) -> PathBuf {
        CONFIG_DIR.join(self.file_name())
    }
}

/// A trait for configs that can be deserialized.
pub trait Config<'de>: Deserialize<'de> {
    /// The [`ConfigFile`] this config corresponds to.
    fn file() -> ConfigFile;
}

/// The services config.
pub mod services {
    use super::*;

    /// Defines a service to update.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
    pub struct ServiceConfig {
        /// An optional name for the service, for easier identification.
        pub name: Option<String>,
        /// The label of the service.
        ///
        /// This is used to identify the target of update requests. Must be unique among all services.
        pub service_label: String,
        /// The container name of the service.
        ///
        /// This is used to identify the Docker container to update. Must match the actual container name.
        pub container_name: String,
        /// The image to update the service from.
        pub image: String,
    }

    /// Defines the services related to updates.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
    pub struct ServicesConfig {
        /// The list of services to update.
        pub services: Vec<ServiceConfig>,
    }

    impl Config<'_> for ServicesConfig {
        fn file() -> ConfigFile {
            ConfigFile::Services
        }
    }
}
