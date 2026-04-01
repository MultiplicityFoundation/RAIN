# Research Homepage Repositioning Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reposition the public R.A.I.N. Lab homepage around a raw research question that triggers an evidence-aware expert panel, without turning the authenticated dashboard into a marketing page.

**Architecture:** Keep the public landing page in `lab_server/static/` and the HTTP contract in `lab_server/app.py`. Add a small `lab_server/research_panel.py` adapter that reuses `james_library/launcher/swarm_orchestrator.py` for multi-specialist question handling and `rain_lab_runtime.py` provenance helpers for evidence labeling. Leave the authenticated React dashboard under `web/src/` functionally unchanged, updating only public metadata surfaces that affect SEO/share cards.

**Tech Stack:** FastAPI, plain HTML/CSS/JS in `lab_server/static/`, Python pytest + FastAPI `TestClient`, existing R.A.I.N. Python orchestration (`run_blackboard_lab`), existing provenance extraction in `rain_lab_runtime.py`.

---

## File Map

- Create: `lab_server/research_panel.py`
  - Builds the research-focused expert panel manifests for raw questions.
  - Normalizes specialist notes into a structured response envelope for the landing page.
- Create: `tests/test_lab_server_homepage.py`
  - Covers the public landing page copy, `/debate` response contract, and metadata alignment.
- Modify: `lab_server/app.py`
  - Keep it as the thin FastAPI layer.
  - Replace the current preset-driven debate wrapper with the structured research-panel endpoint.
- Modify: `lab_server/static/index.html`
  - Replace the coding-agent landing copy with the approved research-panel story and transcript-style result containers.
- Create: `lab_server/static/homepage.css`
  - Serious, scientific visual system for the public page.
- Create: `lab_server/static/homepage.js`
  - Own the question submission flow, loading phases, staged expert-note reveal, and synthesis rendering.
- Modify: `rain_lab_runtime.py`
  - Expose public provenance helpers so the landing-page adapter can label evidence and uncertainty without duplicating logic.
- Modify: `web/index.html`
  - Update title, description, and social metadata to match the new public positioning.
- Modify: `docs/overrides/main.html`
  - Update JSON-LD metadata so docs search/share surfaces stop describing the product as a coding-agent-only runtime.

## Architectural Notes

- Do **not** repurpose `web/src/pages/Dashboard.tsx` into a marketing homepage. That route is part of the authenticated dashboard shell and should remain an application screen.
- The public homepage promise lives in `lab_server/static/index.html`.
- The landing-page backend should return structured JSON so the UI can render expert cards and synthesis instead of dumping one flat text blob.
- Keep the initial implementation non-streaming. Simulate “live debate” with staged reveal in the browser after the structured response arrives. If the product later needs true streaming, add a dedicated `POST /debate/stream` endpoint in a follow-up spec.

---

### Task 1: Lock the Research-Panel API Contract

**Files:**
- Create: `tests/test_lab_server_homepage.py`
- Modify: `lab_server/app.py`
- Modify: `rain_lab_runtime.py`
- Create: `lab_server/research_panel.py`

- [ ] **Step 1: Write the failing API-contract test**

```python
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
```

- [ ] **Step 2: Run the API test to verify it fails**

Run: `pytest tests/test_lab_server_homepage.py::test_debate_endpoint_returns_structured_research_panel -v`

Expected: FAIL because `lab_server.app` still expects `topic`/`preset` and still returns the old `{topic, preset, result}` shape.

- [ ] **Step 3: Implement the structured research-panel adapter**

```python
# rain_lab_runtime.py
def extract_provenance(response_text: str) -> list[ProvenanceItem]:
    return _extract_provenance(response_text)


def score_grounding_confidence(response_text: str, provenance: list[ProvenanceItem]) -> float:
    return _confidence_score(response_text, provenance)
```

```python
# lab_server/research_panel.py
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


def _normalize_panel_note(note: dict[str, str]) -> dict[str, Any]:
    evidence = extract_provenance(note["notes"])
    return {
        "agent_name": note["agent_name"],
        "role": note["role"],
        "content": note["notes"],
        "evidence_sources": [item.source for item in evidence],
        "grounded": bool(evidence),
        "confidence": score_grounding_confidence(note["notes"], evidence),
    }


async def run_research_panel(question: str) -> dict[str, Any]:
    envelope = await run_blackboard_lab(
        query=question,
        manifests=_research_panel_manifests(),
        config=SwarmConfig(rounds=1, temperature=0.25, max_tokens_per_turn=420, max_context_tokens=6_000),
    )

    panel = [_normalize_panel_note(note) for note in envelope["specialist_notes"]]
    synthesis = envelope["synthesized_response"]
    synthesis_evidence = extract_provenance(synthesis)

    return {
        "question": question,
        "panel_title": "Bell Labs-style panel",
        "panel": panel,
        "synthesis": synthesis,
        "synthesis_evidence_sources": [item.source for item in synthesis_evidence],
        "grounded": bool(synthesis_evidence),
        "confidence": score_grounding_confidence(synthesis, synthesis_evidence),
    }
```

```python
# lab_server/app.py
from lab_server.research_panel import run_research_panel


class DebateRequest(BaseModel):
    question: str

    @field_validator("question")
    @classmethod
    def question_not_empty(cls, value: str) -> str:
        value = value.strip()
        if not value:
            raise ValueError("question must not be empty")
        return value[:_MAX_TOPIC_CHARS]


class DebateNote(BaseModel):
    agent_name: str
    role: str
    content: str
    evidence_sources: list[str]
    grounded: bool
    confidence: float


class DebateResponse(BaseModel):
    question: str
    panel_title: str
    panel: list[DebateNote]
    synthesis: str
    synthesis_evidence_sources: list[str]
    grounded: bool
    confidence: float


@app.post("/debate", response_model=DebateResponse)
async def debate(req: DebateRequest) -> DebateResponse:
    return DebateResponse(**(await run_research_panel(req.question)))
```

- [ ] **Step 4: Run the API test to verify it passes**

Run: `pytest tests/test_lab_server_homepage.py::test_debate_endpoint_returns_structured_research_panel -v`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rain_lab_runtime.py lab_server/app.py lab_server/research_panel.py tests/test_lab_server_homepage.py
git commit -m "feat: add research panel homepage response contract"
```

---

### Task 2: Rebuild the Public Landing Page Around the Approved Story

**Files:**
- Modify: `tests/test_lab_server_homepage.py`
- Modify: `lab_server/static/index.html`
- Create: `lab_server/static/homepage.css`
- Create: `lab_server/static/homepage.js`

- [ ] **Step 1: Add a failing landing-page copy test**

```python
from fastapi.testclient import TestClient

import lab_server.app as lab_app


def test_public_homepage_uses_research_panel_positioning() -> None:
    client = TestClient(lab_app.app)
    response = client.get("/")

    assert response.status_code == 200
    assert "Ask a research question. Get a room full of experts." in response.text
    assert "Private by default. Strong claims tied to papers or explicit evidence." in response.text
    assert "Your engineering task or question" not in response.text
    assert "Architecture Review" not in response.text
```

- [ ] **Step 2: Run the landing-page copy test to verify it fails**

Run: `pytest tests/test_lab_server_homepage.py::test_public_homepage_uses_research_panel_positioning -v`

Expected: FAIL because `lab_server/static/index.html` still markets a coding-agent engineering task flow.

- [ ] **Step 3: Implement the new landing-page markup, styles, and client logic**

```html
<!-- lab_server/static/index.html -->
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>R.A.I.N. Lab | Ask a research question. Get a room full of experts.</title>
  <link rel="stylesheet" href="/static/homepage.css" />
</head>
<body>
  <main class="page-shell">
    <section class="hero">
      <p class="eyebrow">R.A.I.N. Lab</p>
      <h1>Ask a research question. Get a room full of experts.</h1>
      <p class="lede">
        R.A.I.N. Lab turns one raw question into a live, evidence-grounded debate across multiple expert perspectives, then distills the strongest explanations, disagreements, and next moves.
      </p>
      <p class="trust-line">Private by default. Strong claims tied to papers or explicit evidence.</p>

      <form id="debate-form" class="question-form">
        <label for="question">Bring one hard question</label>
        <textarea id="question" name="question" maxlength="500" placeholder="What are the strongest competing explanations for fast radio bursts?" required></textarea>
        <button type="submit">Start the debate</button>
      </form>

      <div class="example-prompts">
        <button type="button" data-question="What are the strongest competing explanations for the placebo effect in chronic pain?">Placebo effect</button>
        <button type="button" data-question="Could small language models beat larger ones in narrow scientific workflows?">Small models</button>
        <button type="button" data-question="What is the most plausible mechanism behind fast radio bursts?">Fast radio bursts</button>
      </div>
    </section>

    <section class="value-strip">
      <article>Different perspectives, not one flat answer</article>
      <article>Claims grounded in papers</article>
      <article>Synthesis you can act on</article>
    </section>

    <section class="how-it-works">
      <h2>How it works</h2>
      <ol>
        <li>You ask a hard question</li>
        <li>A panel of experts forms around it</li>
        <li>They debate, cite evidence, and challenge each other</li>
        <li>You get the strongest explanations, open disagreements, and what to test next</li>
      </ol>
    </section>

    <section id="results" class="results" hidden>
      <div class="results-header">
        <p id="status-text">Assembling panel...</p>
      </div>
      <div id="panel-grid" class="panel-grid"></div>
      <article class="synthesis-card">
        <h3>Synthesis</h3>
        <div id="synthesis-body"></div>
        <p id="synthesis-evidence" class="evidence-line"></p>
      </article>
    </section>

    <section class="why-this-exists">
      <h2>Search tools help you find papers.</h2>
      <p>R.A.I.N. Lab helps you think with a room full of experts.</p>
    </section>
  </main>

  <script type="module" src="/static/homepage.js"></script>
</body>
</html>
```

```css
/* lab_server/static/homepage.css */
:root {
  --bg: #f4f0e8;
  --ink: #18212b;
  --muted: #5e6b78;
  --panel: rgba(255, 252, 247, 0.92);
  --border: rgba(24, 33, 43, 0.12);
  --accent: #0f766e;
  --accent-2: #1d4ed8;
  --warning: #b45309;
  --shadow: 0 20px 60px rgba(24, 33, 43, 0.08);
}

body {
  margin: 0;
  font-family: "IBM Plex Sans", "Segoe UI", system-ui, sans-serif;
  background:
    radial-gradient(circle at top, rgba(29, 78, 216, 0.08), transparent 32rem),
    linear-gradient(180deg, #f8f4ed 0%, var(--bg) 100%);
  color: var(--ink);
}

.page-shell {
  width: min(1120px, calc(100vw - 2rem));
  margin: 0 auto;
  padding: 3rem 0 5rem;
}

.hero,
.how-it-works,
.why-this-exists,
.results,
.value-strip article {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 1.5rem;
  box-shadow: var(--shadow);
}

.hero {
  padding: 2.5rem;
}

.eyebrow {
  text-transform: uppercase;
  letter-spacing: 0.18em;
  font-size: 0.78rem;
  color: var(--muted);
}

h1 {
  font-family: "IBM Plex Serif", "Georgia", serif;
  font-size: clamp(2.4rem, 5vw, 4.8rem);
  line-height: 1.02;
  margin: 0.6rem 0 1rem;
}

.question-form textarea,
.question-form button,
.example-prompts button,
.panel-note {
  font: inherit;
}

.panel-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1rem;
}

.panel-note[data-grounded="false"] .evidence-chip {
  background: rgba(180, 83, 9, 0.14);
  color: var(--warning);
}
```

```js
// lab_server/static/homepage.js
const form = document.getElementById('debate-form');
const questionInput = document.getElementById('question');
const results = document.getElementById('results');
const statusText = document.getElementById('status-text');
const panelGrid = document.getElementById('panel-grid');
const synthesisBody = document.getElementById('synthesis-body');
const synthesisEvidence = document.getElementById('synthesis-evidence');

document.querySelectorAll('[data-question]').forEach((button) => {
  button.addEventListener('click', () => {
    questionInput.value = button.dataset.question ?? '';
    questionInput.focus();
  });
});

function renderPanelNote(note) {
  const evidence = note.evidence_sources.length
    ? note.evidence_sources.map((source) => `<span class="evidence-chip">${source}</span>`).join('')
    : '<span class="evidence-chip">Hypothesis / sparse evidence</span>';

  return `
    <article class="panel-note" data-grounded="${note.grounded}">
      <p class="panel-role">${note.role}</p>
      <h3>${note.agent_name}</h3>
      <p class="panel-content">${note.content}</p>
      <div class="panel-evidence">${evidence}</div>
    </article>
  `;
}

async function runDebate(question) {
  statusText.textContent = 'Assembling panel...';
  const response = await fetch('/debate', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ question }),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ detail: 'Unknown error' }));
    throw new Error(error.detail || `Request failed with ${response.status}`);
  }

  return response.json();
}

function revealPanel(data) {
  results.hidden = false;
  panelGrid.innerHTML = '';
  synthesisBody.textContent = '';
  synthesisEvidence.textContent = '';

  const phases = ['Assembling panel...', 'Comparing perspectives...', 'Distilling the synthesis...'];
  let phaseIndex = 0;
  const phaseTimer = window.setInterval(() => {
    phaseIndex = Math.min(phaseIndex + 1, phases.length - 1);
    statusText.textContent = phases[phaseIndex];
  }, 900);

  data.panel.forEach((note, index) => {
    window.setTimeout(() => {
      panelGrid.insertAdjacentHTML('beforeend', renderPanelNote(note));
    }, 550 * (index + 1));
  });

  window.setTimeout(() => {
    window.clearInterval(phaseTimer);
    statusText.textContent = 'Synthesis ready';
    synthesisBody.textContent = data.synthesis;
    synthesisEvidence.textContent = data.synthesis_evidence_sources.length
      ? `Evidence: ${data.synthesis_evidence_sources.join(', ')}`
      : 'Evidence: sparse, treat as provisional.';
  }, 550 * (data.panel.length + 1));
}

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  const question = questionInput.value.trim();
  if (!question) return;

  try {
    const data = await runDebate(question);
    revealPanel(data);
    results.scrollIntoView({ behavior: 'smooth', block: 'start' });
  } catch (error) {
    statusText.textContent = `Something went wrong: ${error.message}`;
    results.hidden = false;
  }
});
```

- [ ] **Step 4: Run the landing-page test to verify it passes**

Run: `pytest tests/test_lab_server_homepage.py::test_public_homepage_uses_research_panel_positioning -v`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add lab_server/static/index.html lab_server/static/homepage.css lab_server/static/homepage.js tests/test_lab_server_homepage.py
git commit -m "feat: reposition public homepage around research debate"
```

---

### Task 3: Align Public Metadata With the New Story

**Files:**
- Modify: `tests/test_lab_server_homepage.py`
- Modify: `web/index.html`
- Modify: `docs/overrides/main.html`

- [ ] **Step 1: Add a failing metadata-alignment test**

```python
def test_public_metadata_matches_research_panel_positioning(repo_root) -> None:
    web_index = (repo_root / "web" / "index.html").read_text(encoding="utf-8")
    docs_override = (repo_root / "docs" / "overrides" / "main.html").read_text(encoding="utf-8")

    assert "Ask a research question. Get a room full of experts." in web_index
    assert "expert panel in a box" in docs_override.lower()
    assert "autonomous coding agent runtime" not in web_index.lower()
```

- [ ] **Step 2: Run the metadata test to verify it fails**

Run: `pytest tests/test_lab_server_homepage.py::test_public_metadata_matches_research_panel_positioning -v`

Expected: FAIL because both files still describe the product as a coding-agent runtime.

- [ ] **Step 3: Update metadata surfaces**

```html
<!-- web/index.html -->
<meta
  name="description"
  content="R.A.I.N. Lab turns one research question into an evidence-grounded panel of experts, then synthesizes the strongest explanations, disagreements, and next moves."
/>
<meta property="og:title" content="R.A.I.N. Lab | Ask a research question. Get a room full of experts." />
<meta
  property="og:description"
  content="A private-by-default expert panel in a box for researchers, independent thinkers, and R&D teams."
/>
<meta name="twitter:title" content="R.A.I.N. Lab | Ask a research question. Get a room full of experts." />
<meta
  name="twitter:description"
  content="A private-by-default expert panel in a box grounded in papers and explicit evidence."
/>
<title>R.A.I.N. Lab | Ask a research question. Get a room full of experts.</title>
```

```html
<!-- docs/overrides/main.html -->
"applicationSubCategory": "Research Reasoning Software",
"description": "A private-by-default expert panel in a box that turns a raw research question into an evidence-grounded debate and synthesis.",
"keywords": [
  "research panel",
  "expert debate",
  "evidence-grounded research AI",
  "independent researcher tools",
  "interdisciplinary reasoning"
],
"featureList": [
  "Multi-expert panel responses for raw research questions",
  "Evidence-aware synthesis with explicit provenance and uncertainty",
  "Private-by-default workflow for researchers and R&D teams"
]
```

- [ ] **Step 4: Run the metadata test to verify it passes**

Run: `pytest tests/test_lab_server_homepage.py::test_public_metadata_matches_research_panel_positioning -v`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add web/index.html docs/overrides/main.html tests/test_lab_server_homepage.py
git commit -m "docs: align public metadata with research panel positioning"
```

---

### Task 4: Full Verification and Release Readiness Check

**Files:**
- Modify: none
- Test: `tests/test_lab_server_homepage.py`
- Test: `tests/test_rain_lab_runtime.py`

- [ ] **Step 1: Run the full homepage test file**

Run: `pytest tests/test_lab_server_homepage.py -v`

Expected: PASS

- [ ] **Step 2: Re-run the provenance regression tests after exposing public helpers**

Run: `pytest tests/test_rain_lab_runtime.py::test_extract_provenance_local_and_web tests/test_rain_lab_runtime.py::test_confidence_score_penalizes_speculation_and_uncertainty -v`

Expected: PASS

- [ ] **Step 3: Smoke-test the public FastAPI app locally**

Run: `@'\nfrom fastapi.testclient import TestClient\nimport lab_server.app as app\nclient = TestClient(app.app)\nprint(client.get('/').status_code)\nprint(client.post('/debate', json={'question': 'What causes fast radio bursts?'}).status_code)\n'@ | python -`

Expected:

```text
200
200
```

- [ ] **Step 4: Manually inspect the staged-reveal UX in a browser**

Run: `uvicorn lab_server.app:app --reload --port 8010`

Expected manual checks:

- Hero shows the approved headline and trust line
- Example prompts populate the question box
- Submitting a question reveals four panel notes plus a synthesis card
- Sparse-evidence notes are visibly labeled instead of looking equally grounded
- The page still works on a narrow mobile viewport

- [ ] **Step 5: Commit the final verification checkpoint**

```bash
git add .
git commit -m "test: verify research homepage repositioning"
```

---

## Plan Self-Review

### Spec Coverage

- Hero, value strip, how-it-works, live debate preview, why-this-exists, use cases, and final CTA are all implemented in Task 2.
- Evidence grounding and explicit uncertainty labeling are implemented in Task 1 and rendered in Task 2.
- Public/private story alignment across metadata surfaces is covered in Task 3.
- The authenticated dashboard remains untouched by design, which preserves the intended architecture boundary.

### Placeholder Scan

- No `TODO`, `TBD`, or “appropriate error handling” placeholders remain.
- All code-changing steps name exact files and show the intended code shape.
- All verification steps include exact commands and expected outcomes.

### Type/Interface Consistency

- Public request contract is `{ question: string }`.
- Public response contract consistently uses `panel`, `synthesis`, `evidence_sources`, `grounded`, and `confidence`.
- The landing page JavaScript and FastAPI models use the same field names.
