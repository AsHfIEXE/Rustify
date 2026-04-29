//! WASM Plugin Host — Loads and runs .wasm plugins in a sandbox.
//!
//! Plugins are .wasm files dropped into the plugins directory.
//! They run in a Wasmtime sandbox with no filesystem or network
//! access unless explicitly allowed through the host API.

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin load error: {0}")]
    LoadError(String),
    #[error("Plugin execution error: {0}")]
    ExecError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Metadata about a loaded plugin.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub path: PathBuf,
    pub loaded: bool,
}

/// The WASM plugin host runtime.
pub struct PluginHost {
    plugins_dir: PathBuf,
    loaded_plugins: Vec<PluginInfo>,
}

impl PluginHost {
    /// Create a new plugin host pointing to the plugins directory.
    pub fn new(plugins_dir: &Path) -> Self {
        Self {
            plugins_dir: plugins_dir.to_path_buf(),
            loaded_plugins: Vec::new(),
        }
    }

    /// Discover all .wasm files in the plugins directory.
    pub fn discover_plugins(&mut self) -> Result<Vec<PluginInfo>, PluginError> {
        self.loaded_plugins.clear();

        if !self.plugins_dir.exists() {
            std::fs::create_dir_all(&self.plugins_dir)?;
            tracing::info!("Created plugins directory: {}", self.plugins_dir.display());
            return Ok(Vec::new());
        }

        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "wasm") {
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".into());

                self.loaded_plugins.push(PluginInfo {
                    name,
                    path,
                    loaded: false,
                });
            }
        }

        tracing::info!("Discovered {} plugins", self.loaded_plugins.len());
        Ok(self.loaded_plugins.clone())
    }

    /// Load and initialize a specific plugin by name.
    ///
    /// TODO: Implement actual Wasmtime loading in Phase 4.
    pub fn load_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        if let Some(plugin) = self.loaded_plugins.iter_mut().find(|p| p.name == name) {
            tracing::info!("Loading plugin: {} from {}", name, plugin.path.display());
            // TODO: Wasmtime Engine + Store + Module + Instance setup
            plugin.loaded = true;
            Ok(())
        } else {
            Err(PluginError::LoadError(format!("Plugin '{name}' not found")))
        }
    }

    /// Get list of all discovered plugins.
    pub fn plugins(&self) -> &[PluginInfo] {
        &self.loaded_plugins
    }
}
