from fastapi.testclient import TestClient

import lab_server.app as lab_app


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
