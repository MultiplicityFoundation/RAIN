//! WASM plugin system for R.A.I.N..
//!
//! Plugins are WebAssembly modules loaded via Extism that can extend
//! R.A.I.N. with custom tools and channels. Enable with `--features plugins-wasm`.

pub mod error;
pub mod host;
pub mod wasm_channel;
pub mod wasm_tool;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Current supported schema version for plugin-provided agent manifests.
pub const AGENT_MANIFEST_SCHEMA_VERSION: u32 = 1;

/// A plugin's declared manifest (loaded from manifest.toml alongside the .wasm).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name (unique identifier)
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Author name or organization
    pub author: Option<String>,
    /// Path to the .wasm file (relative to manifest)
    pub wasm_path: String,
    /// Optional plugin-provided agent manifest paths (relative to manifest)
    #[serde(default)]
    pub agent_manifests: Vec<String>,
    /// Optional discoverability tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional minimum required runtime version
    pub min_runtime_version: Option<String>,
    /// Optional signature used for plugin integrity verification
    pub signature: Option<String>,
    /// Capabilities this plugin provides
    pub capabilities: Vec<PluginCapability>,
    /// Permissions this plugin requests
    #[serde(default)]
    pub permissions: Vec<PluginPermission>,
}

/// What a plugin can do.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    /// Provides one or more tools
    Tool,
    /// Provides a channel implementation
    Channel,
    /// Provides a memory backend
    Memory,
    /// Provides an observer/metrics backend
    Observer,
}

/// Permissions a plugin may request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    /// Can make HTTP requests
    HttpClient,
    /// Can read from the filesystem (within sandbox)
    FileRead,
    /// Can write to the filesystem (within sandbox)
    FileWrite,
    /// Can access environment variables
    EnvRead,
    /// Can read agent memory
    MemoryRead,
    /// Can write agent memory
    MemoryWrite,
}

/// Information about a loaded plugin.
#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub min_runtime_version: Option<String>,
    pub signature: Option<String>,
    pub capabilities: Vec<PluginCapability>,
    pub permissions: Vec<PluginPermission>,
    pub wasm_path: PathBuf,
    pub loaded: bool,
}

/// Discovery metadata for a plugin-provided agent manifest.
#[derive(Debug, Clone, Serialize)]
pub struct AgentPackInfo {
    pub plugin: String,
    pub manifest_path: PathBuf,
    pub schema_version: u32,
    pub tags: Vec<String>,
    pub min_runtime_version: Option<String>,
    pub signature: Option<String>,
}
