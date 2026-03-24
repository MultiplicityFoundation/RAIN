# Plugin Agent Pack Authoring (WASM Plugins)

This guide defines the required manifest fields and layout for plugin-provided agent packs.

## Plugin layout

Each plugin is discovered from `plugins/<plugin-name>/manifest.toml`.

Recommended layout:

```text
plugins/
  example-plugin/
    manifest.toml
    plugin.wasm
    agents/
      researcher.toml
      triage.toml
```

## Required plugin manifest fields

`manifest.toml` must include:

- `name` (string)
- `version` (string)
- `wasm_path` (string, relative path)
- `capabilities` (array)

For agent-pack discovery, include:

- `agent_manifests` (array of relative paths)

Optional trust and compatibility metadata:

- `tags` (array of strings)
- `min_runtime_version` (string)
- `signature` (string)

Example:

```toml
name = "example-plugin"
version = "0.4.0"
wasm_path = "plugin.wasm"
capabilities = ["tool"]
permissions = ["http_client"]

agent_manifests = [
  "agents/researcher.toml",
  "agents/triage.toml",
]

tags = ["agent-pack", "research"]
min_runtime_version = "0.20.0"
signature = "ed25519:..."
```

## Agent manifest requirements

Each file listed in `agent_manifests` is loaded relative to `manifest.toml` and must include:

- `schema_version` (integer)

Compatibility is validated during registration. Agent manifests with newer schema versions than the runtime supports are rejected (fail-fast).

## Trust controls

- Marketplace install from `http(s)` sources is disabled by default.
- Enable `[plugins].marketplace_enabled = true` to allow network installs.
- Plugin requested permissions are checked against `[plugins].allowed_permissions` during install.
