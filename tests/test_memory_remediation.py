import json
from pathlib import Path

from james_library.utilities.memory_remediation import (
    build_remediation_queue,
    execute_remediation_queue,
)


def test_build_remediation_queue_promotes_top_needs_evidence_items(tmp_path: Path) -> None:
    review_queue_path = tmp_path / "memory_review_queue.json"
    review_queue_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-review-queue/v1",
                "items": [
                    {
                        "id": "item-1",
                        "status": "needs_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "replay_failures": [
                            "grounded_turn_ratio below threshold",
                            "actionability_score below threshold",
                        ],
                        "priority_score": 89,
                    },
                    {
                        "id": "item-2",
                        "status": "pending_review",
                        "topic": "ignored",
                        "candidate_memory": "already grounded",
                        "replay_failures": [],
                        "priority_score": 100,
                    },
                ],
            }
        ),
        encoding="utf-8",
    )
    remediation_path = tmp_path / "memory_remediation_queue.json"

    queue = build_remediation_queue(review_queue_path, remediation_path, top_n=5)

    assert len(queue["tasks"]) == 1
    task = queue["tasks"][0]
    assert task["source_review_item_id"] == "item-1"
    assert task["status"] == "pending"
    assert task["task_type"] == "gather_evidence"
    assert "grounded_turn_ratio" in " ".join(task["failure_focus"])
    assert Path(remediation_path).exists()


def test_build_remediation_queue_deduplicates_existing_tasks(tmp_path: Path) -> None:
    review_queue_path = tmp_path / "memory_review_queue.json"
    review_queue_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-review-queue/v1",
                "items": [
                    {
                        "id": "item-1",
                        "status": "needs_evidence",
                        "topic": "resonance in simple everyday language",
                        "candidate_memory": "Resonance is like pushing a swing at the right time.",
                        "replay_failures": ["grounded_turn_ratio below threshold"],
                        "priority_score": 69,
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    remediation_path = tmp_path / "memory_remediation_queue.json"
    remediation_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-remediation-queue/v1",
                "tasks": [
                    {
                        "id": "task-1",
                        "source_review_item_id": "item-1",
                        "status": "pending",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    queue = build_remediation_queue(review_queue_path, remediation_path, top_n=5)

    assert len(queue["tasks"]) == 1
    assert queue["tasks"][0]["source_review_item_id"] == "item-1"


def test_execute_remediation_queue_attaches_evidence_and_promotes_review_item(tmp_path: Path) -> None:
    review_queue_path = tmp_path / "memory_review_queue.json"
    remediation_path = tmp_path / "memory_remediation_queue.json"
    papers_dir = tmp_path / "papers"
    papers_dir.mkdir()
    (papers_dir / "evidence.md").write_text(
        "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
        encoding="utf-8",
    )
    review_queue_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-review-queue/v1",
                "items": [
                    {
                        "id": "item-1",
                        "status": "needs_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "replay_failures": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "provenance": [],
                        "evidence": [],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    remediation_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-remediation-queue/v1",
                "tasks": [
                    {
                        "id": "task-1",
                        "source_review_item_id": "item-1",
                        "status": "pending",
                        "task_type": "gather_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "failure_focus": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "suggested_query": "query",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    review_queue, remediation_queue = execute_remediation_queue(
        review_queue_path,
        remediation_path,
        library_path=tmp_path,
        max_tasks=5,
    )

    assert review_queue["items"][0]["status"] == "pending_review"
    assert review_queue["items"][0]["evidence"]
    assert review_queue["items"][0]["provenance"]
    assert remediation_queue["tasks"][0]["status"] == "completed"
    assert remediation_queue["tasks"][0]["evidence"]


def test_execute_remediation_queue_rejects_when_no_evidence_found(tmp_path: Path) -> None:
    review_queue_path = tmp_path / "memory_review_queue.json"
    remediation_path = tmp_path / "memory_remediation_queue.json"
    review_queue_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-review-queue/v1",
                "items": [
                    {
                        "id": "item-1",
                        "status": "needs_evidence",
                        "topic": "resonance in simple everyday language",
                        "candidate_memory": "A claim with no local support anywhere.",
                        "replay_failures": ["grounded_turn_ratio below threshold"],
                        "priority_score": 69,
                        "provenance": [],
                        "evidence": [],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    remediation_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-remediation-queue/v1",
                "tasks": [
                    {
                        "id": "task-1",
                        "source_review_item_id": "item-1",
                        "status": "pending",
                        "task_type": "gather_evidence",
                        "topic": "resonance in simple everyday language",
                        "candidate_memory": "A claim with no local support anywhere.",
                        "failure_focus": ["grounded_turn_ratio below threshold"],
                        "priority_score": 69,
                        "suggested_query": "query",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    review_queue, remediation_queue = execute_remediation_queue(
        review_queue_path,
        remediation_path,
        library_path=tmp_path,
        max_tasks=5,
    )

    assert review_queue["items"][0]["status"] == "rejected"
    assert review_queue["items"][0]["rejection_reason"] == "no_local_evidence_found"
    assert remediation_queue["tasks"][0]["status"] == "no_evidence"


def test_execute_remediation_queue_ignores_readme_and_logs_as_evidence(tmp_path: Path) -> None:
    review_queue_path = tmp_path / "memory_review_queue.json"
    remediation_path = tmp_path / "memory_remediation_queue.json"
    (tmp_path / "README.md").write_text(
        "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
        encoding="utf-8",
    )
    (tmp_path / "RAIN_LAB_MEETING_LOG.md").write_text(
        "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
        encoding="utf-8",
    )

    review_queue_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-review-queue/v1",
                "items": [
                    {
                        "id": "item-1",
                        "status": "needs_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "replay_failures": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "provenance": [],
                        "evidence": [],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    remediation_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-remediation-queue/v1",
                "tasks": [
                    {
                        "id": "task-1",
                        "source_review_item_id": "item-1",
                        "status": "pending",
                        "task_type": "gather_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "failure_focus": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "suggested_query": "query",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    review_queue, remediation_queue = execute_remediation_queue(
        review_queue_path,
        remediation_path,
        library_path=tmp_path,
        max_tasks=5,
    )

    assert review_queue["items"][0]["status"] == "rejected"
    assert remediation_queue["tasks"][0]["status"] == "no_evidence"


def test_execute_remediation_queue_requires_claim_overlap_from_papers(tmp_path: Path) -> None:
    review_queue_path = tmp_path / "memory_review_queue.json"
    remediation_path = tmp_path / "memory_remediation_queue.json"
    papers_dir = tmp_path / "papers"
    papers_dir.mkdir()
    (papers_dir / "paper.md").write_text(
        "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
        encoding="utf-8",
    )
    review_queue_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-review-queue/v1",
                "items": [
                    {
                        "id": "item-1",
                        "status": "needs_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "replay_failures": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "provenance": [],
                        "evidence": [],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    remediation_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-remediation-queue/v1",
                "tasks": [
                    {
                        "id": "task-1",
                        "source_review_item_id": "item-1",
                        "status": "pending",
                        "task_type": "gather_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "failure_focus": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "suggested_query": "query",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    review_queue, remediation_queue = execute_remediation_queue(
        review_queue_path,
        remediation_path,
        library_path=tmp_path,
        max_tasks=5,
    )

    assert review_queue["items"][0]["status"] == "pending_review"
    assert remediation_queue["tasks"][0]["evidence"][0]["source"] == "papers\\paper.md"


def test_execute_remediation_queue_rejects_generic_term_overlap_without_phrase_support(tmp_path: Path) -> None:
    review_queue_path = tmp_path / "memory_review_queue.json"
    remediation_path = tmp_path / "memory_remediation_queue.json"
    papers_dir = tmp_path / "papers"
    papers_dir.mkdir()
    (papers_dir / "paper.md").write_text(
        "This paper discusses geometric constraints, coherence behavior, and dynamic systems in abstract terms.",
        encoding="utf-8",
    )
    review_queue_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-review-queue/v1",
                "items": [
                    {
                        "id": "item-1",
                        "status": "needs_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "replay_failures": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "provenance": [],
                        "evidence": [],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    remediation_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-remediation-queue/v1",
                "tasks": [
                    {
                        "id": "task-1",
                        "source_review_item_id": "item-1",
                        "status": "pending",
                        "task_type": "gather_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "failure_focus": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "suggested_query": "query",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    review_queue, remediation_queue = execute_remediation_queue(
        review_queue_path,
        remediation_path,
        library_path=tmp_path,
        max_tasks=5,
    )

    assert review_queue["items"][0]["status"] == "rejected"
    assert remediation_queue["tasks"][0]["status"] == "no_evidence"


def test_execute_remediation_queue_rejects_adjacent_physics_overlap_false_positive(tmp_path: Path) -> None:
    review_queue_path = tmp_path / "memory_review_queue.json"
    remediation_path = tmp_path / "memory_remediation_queue.json"
    papers_dir = tmp_path / "papers"
    papers_dir.mkdir()
    (papers_dir / "paper.md").write_text(
        "We propose that physical reality may be fundamentally computational, operating on a finite set of geometric update rules applied to a discrete state space. Continuous spacetime and relativistic quantum fields emerge from coarse-grained dynamics.",
        encoding="utf-8",
    )
    review_queue_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-review-queue/v1",
                "items": [
                    {
                        "id": "item-1",
                        "status": "needs_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "replay_failures": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "provenance": [],
                        "evidence": [],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    remediation_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-remediation-queue/v1",
                "tasks": [
                    {
                        "id": "task-1",
                        "source_review_item_id": "item-1",
                        "status": "pending",
                        "task_type": "gather_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "failure_focus": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "suggested_query": "query",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    review_queue, remediation_queue = execute_remediation_queue(
        review_queue_path,
        remediation_path,
        library_path=tmp_path,
        max_tasks=5,
    )

    assert review_queue["items"][0]["status"] == "rejected"
    assert remediation_queue["tasks"][0]["status"] == "no_evidence"


def test_execute_remediation_queue_rejects_bag_of_words_overlap_without_phrase_match(tmp_path: Path) -> None:
    review_queue_path = tmp_path / "memory_review_queue.json"
    remediation_path = tmp_path / "memory_remediation_queue.json"
    papers_dir = tmp_path / "papers"
    papers_dir.mkdir()
    (papers_dir / "paper.md").write_text(
        "Coherence constraints under the right molecular assembly can guide patterns of acoustic interference.",
        encoding="utf-8",
    )
    review_queue_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-review-queue/v1",
                "items": [
                    {
                        "id": "item-1",
                        "status": "needs_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "replay_failures": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "provenance": [],
                        "evidence": [],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    remediation_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-remediation-queue/v1",
                "tasks": [
                    {
                        "id": "task-1",
                        "source_review_item_id": "item-1",
                        "status": "pending",
                        "task_type": "gather_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "failure_focus": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "suggested_query": "query",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    review_queue, remediation_queue = execute_remediation_queue(
        review_queue_path,
        remediation_path,
        library_path=tmp_path,
        max_tasks=5,
    )

    assert review_queue["items"][0]["status"] == "rejected"
    assert remediation_queue["tasks"][0]["status"] == "no_evidence"


def test_execute_remediation_queue_accepts_supportive_paraphrase_with_anchor_phrase(tmp_path: Path) -> None:
    review_queue_path = tmp_path / "memory_review_queue.json"
    remediation_path = tmp_path / "memory_remediation_queue.json"
    papers_dir = tmp_path / "papers"
    papers_dir.mkdir()
    (papers_dir / "paper.md").write_text(
        "Acoustic interference can guide assembly of molecules when coherence constraints are satisfied and the field remains stable.",
        encoding="utf-8",
    )
    review_queue_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-review-queue/v1",
                "items": [
                    {
                        "id": "item-1",
                        "status": "needs_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "replay_failures": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "provenance": [],
                        "evidence": [],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    remediation_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-remediation-queue/v1",
                "tasks": [
                    {
                        "id": "task-1",
                        "source_review_item_id": "item-1",
                        "status": "pending",
                        "task_type": "gather_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "failure_focus": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "suggested_query": "query",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    review_queue, remediation_queue = execute_remediation_queue(
        review_queue_path,
        remediation_path,
        library_path=tmp_path,
        max_tasks=5,
    )

    assert review_queue["items"][0]["status"] == "pending_review"
    assert remediation_queue["tasks"][0]["status"] == "completed"


def test_execute_remediation_queue_rejects_contradictory_evidence(tmp_path: Path) -> None:
    review_queue_path = tmp_path / "memory_review_queue.json"
    remediation_path = tmp_path / "memory_remediation_queue.json"
    papers_dir = tmp_path / "papers"
    papers_dir.mkdir()
    (papers_dir / "paper.md").write_text(
        "Acoustic interference cannot guide molecular assembly because coherence constraints break down under noise.",
        encoding="utf-8",
    )
    review_queue_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-review-queue/v1",
                "items": [
                    {
                        "id": "item-1",
                        "status": "needs_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "replay_failures": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "provenance": [],
                        "evidence": [],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    remediation_path.write_text(
        json.dumps(
            {
                "schema_version": "rain-memory-remediation-queue/v1",
                "tasks": [
                    {
                        "id": "task-1",
                        "source_review_item_id": "item-1",
                        "status": "pending",
                        "task_type": "gather_evidence",
                        "topic": "Could acoustic interference patterns guide molecular assembly the way DNA guides cell growth?",
                        "candidate_memory": "Acoustic interference patterns can guide molecular assembly under the right coherence constraints.",
                        "failure_focus": ["grounded_turn_ratio below threshold"],
                        "priority_score": 89,
                        "suggested_query": "query",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    review_queue, remediation_queue = execute_remediation_queue(
        review_queue_path,
        remediation_path,
        library_path=tmp_path,
        max_tasks=5,
    )

    assert review_queue["items"][0]["status"] == "rejected"
    assert remediation_queue["tasks"][0]["status"] == "no_evidence"
