import asyncio
import re
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

import lab_server.app as lab_app
import lab_server.research_panel as research_panel


def test_debate_endpoint_returns_structured_research_panel(monkeypatch) -> None:
    async def fake_run_research_panel(question: str) -> dict[str, object]:
        return {
            "question": question,
            "panel_title": "Bell Labs-style panel",
            "panel": [
                {
                    "agent_name": "Mechanism Hunter",
                    "role": "Mechanistic modeler",
                    "content": "The leading explanation points to magnetar activity [from web: Example Paper].",
                    "evidence_sources": ["Example Paper"],
                    "grounded": True,
                    "confidence": 0.86,
                }
            ],
            "synthesis": "Most evidence clusters around the magnetar explanation [from web: Example Paper].",
            "synthesis_evidence_sources": ["Example Paper"],
            "grounded": True,
            "confidence": 0.91,
        }

    monkeypatch.setattr(lab_app, "run_research_panel", fake_run_research_panel)
    client = TestClient(lab_app.app)

    response = client.post("/debate", json={"question": "What causes fast radio bursts?"})
    body = response.json()

    assert response.status_code == 200
    assert body["question"] == "What causes fast radio bursts?"
    assert body["panel_title"] == "Bell Labs-style panel"
    assert body["panel"][0]["agent_name"] == "Mechanism Hunter"
    assert body["panel"][0]["grounded"] is True
    assert body["synthesis"]
    assert body["grounded"] is True


@pytest.mark.parametrize(
    ("error_message", "expected_detail"),
    [
        (
            "R.A.I.N. runtime config error: file not found: missing.toml",
            "R.A.I.N. runtime config error: file not found: missing.toml",
        ),
        (
            "The operation was canceled.",
            "R.A.I.N. runtime canceled: the operation was canceled. Retry and verify LM Studio is running with a loaded model.",
        ),
        (
            "budget exceeded",
            "R.A.I.N. runtime error: unable to generate response.",
        ),
    ],
)
def test_debate_endpoint_translates_research_panel_failures(monkeypatch, error_message: str, expected_detail: str) -> None:
    async def fake_run_research_panel(question: str) -> dict[str, object]:
        raise RuntimeError(error_message)

    monkeypatch.setattr(lab_app, "run_research_panel", fake_run_research_panel)
    client = TestClient(lab_app.app, raise_server_exceptions=False)

    response = client.post("/debate", json={"question": "What causes fast radio bursts?"})

    assert response.status_code == 500
    assert response.json() == {"detail": expected_detail}


def test_debate_endpoint_translates_cancelled_error(monkeypatch) -> None:
    async def fake_run_research_panel(question: str) -> dict[str, object]:
        raise asyncio.CancelledError()

    monkeypatch.setattr(lab_app, "run_research_panel", fake_run_research_panel)
    client = TestClient(lab_app.app, raise_server_exceptions=False)

    response = client.post("/debate", json={"question": "What causes fast radio bursts?"})

    assert response.status_code == 500
    assert response.json() == {
        "detail": "R.A.I.N. runtime canceled: the operation was canceled. Retry and verify LM Studio is running with a loaded model."
    }


def test_normalize_panel_note_extracts_grounding_metadata() -> None:
    normalized = research_panel._normalize_panel_note(
        {
            "agent_name": "Mechanism Hunter",
            "role": "Mechanistic modeler",
            "notes": 'Quoted support "coherent oscillatory inputs reduce cost" [from Local Paper.md] [from web: Example Site].',
        }
    )

    assert normalized["agent_name"] == "Mechanism Hunter"
    assert normalized["role"] == "Mechanistic modeler"
    assert normalized["grounded"] is True
    assert normalized["evidence_sources"] == ["Local Paper.md", "Example Site"]
    assert normalized["confidence"] == 0.71


def test_run_research_panel_normalizes_synthesis_provenance(monkeypatch) -> None:
    async def fake_run_blackboard_lab(**kwargs) -> dict[str, object]:
        return {
            "specialist_notes": [
                {
                    "agent_name": "Mechanism Hunter",
                    "role": "Mechanistic modeler",
                    "notes": "Magnetars remain strongest [from web: Example Paper].",
                }
            ],
            "synthesized_response": '  Synthesis with "useful quoted support" [from Paper A.md] [from web: Example Paper].  ',
        }

    monkeypatch.setattr(research_panel, "run_blackboard_lab", fake_run_blackboard_lab)

    result = asyncio.run(research_panel.run_research_panel("What causes fast radio bursts?"))

    assert result["question"] == "What causes fast radio bursts?"
    assert result["panel"][0]["evidence_sources"] == ["Example Paper"]
    assert result["synthesis"] == 'Synthesis with "useful quoted support" [from Paper A.md] [from web: Example Paper].'
    assert result["synthesis_evidence_sources"] == ["Paper A.md", "Example Paper"]
    assert result["grounded"] is True
    assert result["confidence"] == 0.71


def test_homepage_shows_research_panel_positioning_and_no_longer_shows_coding_agent_copy() -> None:
    client = TestClient(lab_app.app)

    response = client.get("/")
    html = response.text

    assert response.status_code == 200
    assert "Ask a research question. Get a room full of experts." in html
    assert "Private by default. Strong claims tied to papers or explicit evidence." in html
    assert "expert panel in a box" in html
    assert "Different perspectives, not one flat answer" in html
    assert "Search tools help you find papers. R.A.I.N. Lab helps you think with a room full of experts." in html
    assert 'placeholder="For example: How should I interpret these conflicting findings, and what evidence separates the leading explanations?"' in html
    assert 'data-question="How should I interpret these conflicting findings, and what evidence separates the leading explanations?"' in html
    assert re.search(
        r'<button type="button" class="example-prompts__item" data-question="How should I interpret these conflicting findings, and what evidence separates the leading explanations\?">\s*Conflicting findings\s*</button>',
        html,
    )
    assert "The local-first autonomous coding agent for Rust, Python, and hardware teams" not in html
    assert "Your engineering task or question" not in html
    assert "Run the task ->" not in html
    assert "fast radio bursts" not in html


def test_public_metadata_surfaces_reflect_research_panel_positioning() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    web_index = (repo_root / "web" / "index.html").read_text(encoding="utf-8")
    docs_override = (repo_root / "docs" / "overrides" / "main.html").read_text(encoding="utf-8")

    assert "Ask a research question. Get a room full of experts." in web_index
    assert "expert panel in a box" in web_index
    assert "Private by default. Strong claims tied to papers or explicit evidence." in web_index
    assert "Researchers, independent thinkers, and R&D teams" in web_index
    assert "autonomous coding agent runtime" not in web_index

    assert "Research Reasoning Software" in docs_override
    assert "expert panel in a box" in docs_override
    assert "evidence-grounded debate" in docs_override
    assert "researchers, independent thinkers, and R&D teams" in docs_override
    assert "autonomous coding agents" not in docs_override
