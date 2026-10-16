/************************************************
* File: config.rs
* Author: Michal Švrček
*
* Project-specific RepoDoctor configuration.
*
* ver. 0.5.0
*************************************************/

use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub files: FileConfig,
    pub release: ReleaseConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct FileConfig {
    pub large_file_mb: u64,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ReleaseConfig {
    pub require_license: bool,
    pub warn_existing_tag: bool,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self { large_file_mb: 10 }
    }
}

impl Default for ReleaseConfig {
    fn default() -> Self {
        Self {
            require_license: false,
            warn_existing_tag: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            files: FileConfig::default(),
            release: ReleaseConfig::default(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        let file = path.join("repodoctor.toml");

        if !file.is_file() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&file).map_err(|e| format!("Cannot read config: {e}"))?;

        let config: Self =
            toml::from_str(&content).map_err(|e| format!("Invalid repodoctor.toml: {e}"))?;

        if config.files.large_file_mb == 0 {
            return Err("large_file_mb must be greater than zero".into());
        }

        if config.files.large_file_mb > u64::MAX / (1024 * 1024) {
            return Err("large_file_mb is too large".into());
        }

        Ok(config)
    }
}
