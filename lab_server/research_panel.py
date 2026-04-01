from __future__ import annotations

from typing import Any

from james_library.launcher.swarm_orchestrator import (
    AgentIdentity,
    AgentManifest,
    AgentMemoryRouting,
    AgentToolScope,
    SwarmConfig,
    run_blackboard_lab,
)
from rain_lab_runtime import extract_provenance, score_grounding_confidence

_PANEL_TITLE = "Bell Labs-style panel"


def _research_panel_manifests() -> list[AgentManifest]:
    specs = [
        (
            "mechanism-hunter",
            "Mechanism Hunter",
            "Mechanistic modeler",
            "Find the strongest causal explanation. Cite every strong claim as [from filename.md] or [from web: source]. Mark weak leaps as [HYPOTHESIS].",
        ),
        (
            "evidence-auditor",
            "Evidence Auditor",
            "Evidence skeptic",
            "Separate what is actually supported from what is merely plausible. Demand citations and label gaps clearly.",
        ),
        (
            "adjacent-scout",
            "Adjacent Scout",
            "Cross-disciplinary scout",
            "Bring in overlooked adjacent fields, analogies, and literature. Cite specific sources whenever possible.",
        ),
        (
            "experiment-designer",
            "Experiment Designer",
            "Experiment strategist",
            "Turn the debate into concrete tests, measurements, or next readings. Cite evidence for why each next step matters.",
        ),
    ]
    manifests: list[AgentManifest] = []
    for agent_id, display_name, role, system_prompt in specs:
        manifests.append(
            AgentManifest(
                schema_version="1.0",
                identity=AgentIdentity(
                    agent_id=agent_id,
                    display_name=display_name,
                    role=role,
                    system_prompt=system_prompt,
                ),
                tools=AgentToolScope(allowed=["web_search", "paper_search"]),
                memory=AgentMemoryRouting(categories=["research"]),
            )
        )
    return manifests


def _normalize_panel_note(note: dict[str, Any]) -> dict[str, Any]:
    content = str(note.get("notes", "")).strip()
    evidence = extract_provenance(content)
    return {
        "agent_name": str(note.get("agent_name", "")),
        "role": str(note.get("role", "")),
        "content": content,
        "evidence_sources": [item.source for item in evidence],
        "grounded": bool(evidence),
        "confidence": score_grounding_confidence(content, evidence),
    }


async def run_research_panel(question: str) -> dict[str, Any]:
    envelope = await run_blackboard_lab(
        query=question,
        manifests=_research_panel_manifests(),
        config=SwarmConfig(
            rounds=1,
            temperature=0.25,
            max_tokens_per_turn=420,
            max_context_tokens=6_000,
        ),
    )

    panel = [_normalize_panel_note(note) for note in envelope["specialist_notes"]]
    synthesis = str(envelope["synthesized_response"]).strip()
    synthesis_evidence = extract_provenance(synthesis)

    return {
        "question": question,
        "panel_title": _PANEL_TITLE,
        "panel": panel,
        "synthesis": synthesis,
        "synthesis_evidence_sources": [item.source for item in synthesis_evidence],
        "grounded": bool(synthesis_evidence),
        "confidence": score_grounding_confidence(synthesis, synthesis_evidence),
    }
