"""Session-scoped cost tracking and hard budget enforcement for Python loops."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
import sqlite3


@dataclass(frozen=True)
class ModelPrice:
    input_per_million: float
    output_per_million: float


@dataclass(frozen=True)
class BudgetExceededError(RuntimeError):
    total_spent: float
    limit: float

    def __str__(self) -> str:
        return (
            f"Budget limit reached (${self.limit:.2f}). "
            f"Total spent: ${self.total_spent:.2f}."
        )


_DEFAULT_PRICE_MAP: dict[str, ModelPrice] = {
    "openai/gpt-4o": ModelPrice(5.0, 15.0),
    "gpt-4o": ModelPrice(5.0, 15.0),
    "openai/o1-preview": ModelPrice(15.0, 60.0),
    "o1-preview": ModelPrice(15.0, 60.0),
    "anthropic/claude-3-5-sonnet": ModelPrice(3.0, 15.0),
    "claude-3-5-sonnet": ModelPrice(3.0, 15.0),
    "anthropic/claude-3.5-sonnet": ModelPrice(3.0, 15.0),
    "claude-3.5-sonnet": ModelPrice(3.0, 15.0),
    "anthropic/claude-sonnet-4-20250514": ModelPrice(3.0, 15.0),
    "claude-sonnet-4-20250514": ModelPrice(3.0, 15.0),
    "openai/gpt-4o-mini": ModelPrice(0.15, 0.60),
    "gpt-4o-mini": ModelPrice(0.15, 0.60),
}


class CostMonitor:
    """Track per-session API spend and persist it for resume-safe orchestration."""

    def __init__(
        self,
        *,
        session_id: str,
        workspace_root: str | Path,
        db_path: str | Path | None = None,
        price_map: dict[str, ModelPrice] | None = None,
    ) -> None:
        self.session_id = session_id
        self.workspace_root = Path(workspace_root).resolve()
        self.db_path = Path(db_path) if db_path is not None else (
            self.workspace_root / "meeting_archives" / "episodic_costs.sqlite3"
        )
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self._price_map = {
            key.lower(): value for key, value in (price_map or _DEFAULT_PRICE_MAP).items()
        }
        self._ensure_schema()
        self.session_cost = self._load_session_cost()

    def update_cost(self, model_name: str, prompt_tokens: int, completion_tokens: int) -> float:
        """Calculate, persist, and accumulate the cost of one turn."""

        price = self._resolve_price(model_name)
        delta = (
            (max(0, int(prompt_tokens)) / 1_000_000.0) * price.input_per_million
            + (max(0, int(completion_tokens)) / 1_000_000.0) * price.output_per_million
        )
        self._persist_event(
            model_name=model_name,
            prompt_tokens=max(0, int(prompt_tokens)),
            completion_tokens=max(0, int(completion_tokens)),
            total_cost_usd=delta,
        )
        self.session_cost += delta
        return delta

    def check_budget(self, limit: float) -> None:
        """Raise when the running session total reaches or exceeds the budget."""

        if self.session_cost >= float(limit):
            raise BudgetExceededError(total_spent=self.session_cost, limit=float(limit))

    def _resolve_price(self, model_name: str) -> ModelPrice:
        normalized = model_name.strip().lower()
        return self._price_map.get(normalized, ModelPrice(0.0, 0.0))

    def _connect(self) -> sqlite3.Connection:
        connection = sqlite3.connect(self.db_path)
        connection.execute("PRAGMA journal_mode=WAL")
        return connection

    def _ensure_schema(self) -> None:
        with self._connect() as connection:
            connection.execute(
                """
                CREATE TABLE IF NOT EXISTS cost_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    model_name TEXT NOT NULL,
                    prompt_tokens INTEGER NOT NULL,
                    completion_tokens INTEGER NOT NULL,
                    total_cost_usd REAL NOT NULL,
                    created_at TEXT NOT NULL
                )
                """
            )
            connection.execute(
                "CREATE INDEX IF NOT EXISTS idx_cost_events_session_id ON cost_events(session_id)"
            )

    def _load_session_cost(self) -> float:
        with self._connect() as connection:
            row = connection.execute(
                "SELECT COALESCE(SUM(total_cost_usd), 0.0) FROM cost_events WHERE session_id = ?",
                (self.session_id,),
            ).fetchone()
        return float(row[0] if row else 0.0)

    def _persist_event(
        self,
        *,
        model_name: str,
        prompt_tokens: int,
        completion_tokens: int,
        total_cost_usd: float,
    ) -> None:
        with self._connect() as connection:
            connection.execute(
                """
                INSERT INTO cost_events (
                    session_id,
                    model_name,
                    prompt_tokens,
                    completion_tokens,
                    total_cost_usd,
                    created_at
                ) VALUES (?, ?, ?, ?, ?, ?)
                """,
                (
                    self.session_id,
                    model_name,
                    prompt_tokens,
                    completion_tokens,
                    float(total_cost_usd),
                    datetime.now(timezone.utc).isoformat(),
                ),
            )


__all__ = ["BudgetExceededError", "CostMonitor", "ModelPrice"]
