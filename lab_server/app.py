"""R.A.I.N. Lab — hosted web server for lab.vers3dynamics.com.

Wraps rain_lab_runtime.run_rain_lab behind a minimal FastAPI interface.
Deploy to maritime.sh via GitHub. Configure via environment variables:

  LM_STUDIO_BASE_URL  — LLM API base URL  (e.g. https://api.openai.com/v1)
  LM_STUDIO_API_KEY   — API key
  LM_STUDIO_MODEL     — model name        (e.g. gpt-4o-mini)
"""

from __future__ import annotations

import sys
from pathlib import Path

# Make repo root importable when running from lab_server/ or from repo root.
_REPO_ROOT = Path(__file__).resolve().parent.parent
if str(_REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT))

from fastapi import FastAPI
from fastapi.responses import FileResponse, HTMLResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel, field_validator

from lab_server.research_panel import run_research_panel

_STATIC_DIR = Path(__file__).parent / "static"
_MAX_TOPIC_CHARS = 500

app = FastAPI(title="R.A.I.N. Lab", docs_url=None, redoc_url=None)
app.mount("/static", StaticFiles(directory=str(_STATIC_DIR)), name="static")


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


@app.get("/health")
def health() -> dict:
    return {"status": "ok"}


@app.get("/", response_class=HTMLResponse)
def index() -> FileResponse:
    return FileResponse(_STATIC_DIR / "index.html")


@app.post("/debate", response_model=DebateResponse)
async def debate(req: DebateRequest) -> DebateResponse:
    return DebateResponse(**(await run_research_panel(req.question)))
