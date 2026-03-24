use crate::memory::MemoryCategory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgentManifest {
    pub schema_version: String,
    pub identity: IdentitySection,
    pub tools: ToolScope,
    #[serde(default)]
    pub memory: Option<MemoryRouting>,
    #[serde(default)]
    pub rag: Option<RagRouting>,
    #[serde(default)]
    pub orchestration: Option<OrchestrationHints>,
    #[serde(default)]
    pub provider_defaults: Option<ProviderDefaults>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct IdentitySection {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ToolScope {
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub session_scope: SessionScope,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionScope {
    #[default]
    Current,
    CrossSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MemoryRouting {
    #[serde(default)]
    pub recall_limit: Option<usize>,
    #[serde(default)]
    pub min_relevance_score: Option<f64>,
    #[serde(default)]
    pub category: Option<MemoryCategory>,
    #[serde(default)]
    pub session_scope: SessionScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RagRouting {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub max_chunks: Option<usize>,
    #[serde(default)]
    pub min_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OrchestrationHints {
    pub autonomy: AutonomyMode,
    #[serde(default)]
    pub parallel_tools: Option<bool>,
    #[serde(default)]
    pub max_iterations: Option<usize>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyMode {
    Supervised,
    Full,
    ReadOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct ProviderDefaults {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub temperature: Option<f64>,
}
