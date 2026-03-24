//! Named tool profiles and expansion helpers.

use std::collections::{HashMap, HashSet};

/// Built-in profile registry mapping profile names to canonical tool selectors.
fn built_in_profiles() -> HashMap<&'static str, &'static [&'static str]> {
    const ACOUSTIC_TOOLS: &[&str] = &["web_fetch", "web_search", "http_request", "text_browser"];
    const ARCHITECT_TOOLS: &[&str] = &[
        "file_read",
        "glob_search",
        "content_search",
        "pdf_read",
        "knowledge",
    ];
    const CODER_TOOLS: &[&str] = &[
        "shell",
        "file_read",
        "file_write",
        "file_edit",
        "glob_search",
        "content_search",
        "git_operations",
    ];
    const RESEARCH_TOOLS: &[&str] = &["web_search", "web_fetch", "http_request"];
    const MCP_ALL: &[&str] = &["mcp:*"];
    HashMap::from([
        ("acoustic_tools", ACOUSTIC_TOOLS),
        ("architect_tools", ARCHITECT_TOOLS),
        ("coder_tools", CODER_TOOLS),
        ("research_tools", RESEARCH_TOOLS),
        ("mcp_all", MCP_ALL),
    ])
}

/// Expand named profiles into a flat selector list.
pub fn expand_profiles(profiles: &[String]) -> HashSet<String> {
    let registry = built_in_profiles();
    let mut expanded = HashSet::new();

    for profile in profiles {
        let key = profile.trim();
        if key.is_empty() {
            continue;
        }
        if let Some(entries) = registry.get(key) {
            expanded.extend(entries.iter().map(|entry| (*entry).to_string()));
        } else {
            // Unknown profiles are treated as explicit selectors so users can
            // provide direct tool names/patterns in profile slots when desired.
            expanded.insert(key.to_string());
        }
    }

    expanded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_profiles_expands_known_profile() {
        let expanded = expand_profiles(&["coder_tools".to_string()]);
        assert!(expanded.contains("shell"));
        assert!(expanded.contains("file_edit"));
    }

    #[test]
    fn expand_profiles_keeps_unknown_profile_as_selector() {
        let expanded = expand_profiles(&["mcp:browser/*".to_string()]);
        assert!(expanded.contains("mcp:browser/*"));
    }
}
