# Python package map

The repository still keeps thin root-level wrappers such as `rain_lab.py` and `deploy.py` so existing commands keep working, but the implementation now lives under the `james_library/` package.

## Package layout

```text
james_library/
├── launcher/
│   ├── rain_lab.py
│   ├── meeting_workflow.py
│   └── swarm_orchestrator.py
├── bootstrap/
│   ├── rain_preflight_check.py
│   ├── rain_first_run.py
│   └── deploy.py
├── services/
│   ├── openclaw_service.py
│   └── external_integrations.py
└── utilities/
    ├── tools.py
    ├── truth_layer.py
    └── library_compiler.py
```

## Domain boundaries

- `james_library.launcher`: entrypoint orchestration, meeting state, and peer-review swarm flow.
- `james_library.bootstrap`: environment checks, onboarding helpers, and service installation commands.
- `james_library.services`: long-running service supervision and external research integrations.
- `james_library.utilities`: reusable helper modules that can be imported by launchers, services, or tests.

## Placement rules for new Python code

- Add new CLI behavior to the relevant package module first, then keep the root wrapper as a compatibility shim if a legacy command already exists.
- Put reusable logic in `services/` or `utilities/` instead of embedding it in root scripts.
- Prefer tests that import `james_library.*` modules directly; reserve root-wrapper checks for backward-compatibility smoke coverage.
- When a wrapper is no longer needed, remove it in a focused follow-up after downstream command usage has been updated.
