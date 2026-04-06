//! MCP tool resolution and filtering.
//!
//! Handles MCP prefixed-tool name expansion, alias generation, and
//! allowlist/denylist matching against agent config.

use crate::config::AgentConfig;
use crate::tools::expand_profiles;

/// Case-insensitive glob match: `*` matches any sequence of characters.
pub(crate) fn glob_match_ci(pattern: &str, name: &str) -> bool {
    match pattern.find('*') {
        None => pattern.eq_ignore_ascii_case(name),
        Some(star) => {
            let prefix = &pattern[..star];
            let suffix = &pattern[star + 1..];
            name.len() >= prefix.len() + suffix.len()
                && name[..prefix.len()].eq_ignore_ascii_case(prefix)
                && name[name.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
        }
    }
}

/// Expand an MCP prefixed tool name (`namespace__tool`) into all matching aliases.
pub(crate) fn mcp_aliases(prefixed_name: &str) -> Vec<String> {
    let mut out = vec![prefixed_name.to_string()];
    if let Some((namespace, tool)) = prefixed_name.split_once("__") {
        out.push(format!("mcp:{namespace}/{tool}"));
        out.push(format!("mcp:{namespace}/*"));
        out.push("mcp:*".to_string());
    }
    out
}

/// Return true when `selector` (a glob pattern) matches any alias of the
/// given MCP prefixed tool name.
pub(crate) fn mcp_selector_match(selector: &str, prefixed_name: &str) -> bool {
    let selector = selector.trim();
    !selector.is_empty()
        && mcp_aliases(prefixed_name)
            .iter()
            .any(|alias| glob_match_ci(selector, alias))
}

/// Check whether a prefixed MCP tool name is permitted by the agent config's
/// allowlist/denylist and tool profiles.
pub(crate) fn mcp_tool_enabled(prefixed_name: &str, cfg: &AgentConfig) -> bool {
    let mut allow = expand_profiles(&cfg.tool_profiles);
    allow.extend(
        cfg.tool_allowlist
            .iter()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty()),
    );
    let deny: Vec<&str> = cfg
        .tool_denylist
        .iter()
        .map(String::as_str)
        .filter(|v| !v.trim().is_empty())
        .collect();

    if cfg.strict_tool_allowlist && allow.is_empty() {
        return false;
    }

    let allowed = allow.is_empty() || allow.iter().any(|s| mcp_selector_match(s, prefixed_name));
    let denied = deny.iter().any(|s| mcp_selector_match(s, prefixed_name));
    allowed && !denied
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_match_ci_exact() {
        assert!(glob_match_ci("foo", "FOO"));
        assert!(glob_match_ci("foo", "foo"));
        assert!(!glob_match_ci("foo", "bar"));
    }

    #[test]
    fn glob_match_ci_wildcard_prefix() {
        assert!(glob_match_ci("*.txt", "foo.txt"));
        assert!(glob_match_ci("*.txt", "FOO.TXT"));
        assert!(!glob_match_ci("*.txt", "foo.csv"));
    }

    #[test]
    fn glob_match_ci_wildcard_substring() {
        assert!(glob_match_ci("foo*bar", "FOO123bar"));
        assert!(glob_match_ci("foo*bar", "fooBazbar")); // suffix is case-insensitive
        assert!(!glob_match_ci("foo*bar", "foobaz")); // missing suffix "bar"
    }

    #[test]
    fn mcp_aliases_no_namespace() {
        let aliases = mcp_aliases("shell");
        assert_eq!(aliases, vec!["shell"]);
    }

    #[test]
    fn mcp_aliases_with_namespace() {
        let aliases = mcp_aliases("github__pr_list");
        assert_eq!(
            aliases,
            vec![
                "github__pr_list",
                "mcp:github/pr_list",
                "mcp:github/*",
                "mcp:*",
            ]
        );
    }

    #[test]
    fn mcp_selector_match_empty_is_false() {
        assert!(!mcp_selector_match("", "shell"));
        assert!(!mcp_selector_match("   ", "shell"));
    }

    #[test]
    fn mcp_selector_match_wildcard() {
        assert!(mcp_selector_match("mcp:*", "github__pr"));
        assert!(mcp_selector_match("mcp:github/*", "github__pr"));
        assert!(mcp_selector_match("github__*", "github__pr_list"));
    }
}
