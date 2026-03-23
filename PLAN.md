# Maintainability Improvement Plan

## Goal
Decompose the two largest files (`loop_.rs` at 5,973 lines, `config/schema/mod.rs` at 6,654 lines) into focused sub-modules, following the existing extraction patterns already established in the codebase.

## Principles
- Follow CLAUDE.md: KISS, SRP, small reversible changes, one concern per PR
- Follow existing patterns: `tool_call_parser.rs` and `tool_filter.rs` were already extracted from `loop_.rs`; `config_impl.rs`, `root_config.rs`, `ops.rs` were already extracted from schema
- Re-export from parent module to preserve public API — zero breaking changes
- Move tests alongside the code they test

---

## Phase 1: Extract tool execution from loop_.rs

**Target**: `src/agent/tool_execution.rs` (~270 production lines + ~800 test lines)

**What moves:**
- `ToolExecutionOutcome` struct (lines 353-358)
- `find_tool()` (lines 96-98)
- `maybe_inject_channel_delivery_defaults()` (lines 165-257)
- `execute_one_tool()` (lines 259-351)
- `should_execute_tools_in_parallel()` (lines 360-385)
- `execute_tools_parallel()` (lines 387-410)
- `execute_tools_sequential()` (lines 412-436)
- Related tests: parallel execution, cron delivery defaults, deduplication, activated tools

**Why this first**: Self-contained, clear boundary, follows the pattern of prior extractions.

## Phase 2: Extract channel configs from config/schema/mod.rs

**Target**: `src/config/schema/channels.rs` (~904 lines)

**What moves:**
- All 20+ channel config structs (TelegramConfig, DiscordConfig, SlackConfig, etc.)
- Their `Default` impls
- Their `ChannelConfig` trait impls
- The `ChannelsConfig` struct and its impl

**Why second**: Largest single-concern section, all structs follow the same pattern, zero coupling to other config sections.

## Phase 3: Extract proxy runtime from config/schema/mod.rs

**Target**: `src/config/schema/proxy.rs` (~706 lines)

**What moves:**
- `ProxyScope` enum
- `ProxyConfig` struct + impl
- `RuntimeProxyState` / `RuntimeProxyStateHandle`
- All proxy management functions (task-local state, client caching, etc.)
- Related constants

**Why third**: Self-contained concern with its own runtime state management, clear boundary.

## Phase 4: Extract security configs from config/schema/mod.rs

**Target**: `src/config/schema/security.rs` (~758 lines)

**What moves:**
- `OtpConfig`, `EstopConfig`, `NevisConfig`
- `SandboxConfig`, `ResourceLimitsConfig`
- `AuditConfig`
- Social platform configs (Twitter, Reddit, Bluesky, Nostr, Notion, Jira)
- Their `Default` impls

**Why fourth**: Completes the schema decomposition for the three largest sections.

---

## Expected Impact
- `loop_.rs`: ~5,973 → ~4,900 lines (Phase 1)
- `config/schema/mod.rs`: ~6,654 → ~4,286 lines (Phases 2-4)
- No public API changes
- Each phase is independently revertible
