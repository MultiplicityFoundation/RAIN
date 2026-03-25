from pathlib import Path


PRIMARY_DOC_EXPECTATIONS = {
    "README.md": ["INSTALL_RAIN.cmd", "./install.sh", "python rain_lab.py"],
    "START_HERE.md": ["INSTALL_RAIN.cmd", "./install.sh", "python rain_lab.py"],
    "docs/index.md": ["INSTALL_RAIN.cmd", "./install.sh", "python rain_lab.py"],
    "docs/getting-started/README.md": ["INSTALL_RAIN.cmd", "./install.sh", "python rain_lab.py"],
    "docs/one-click-bootstrap.md": ["INSTALL_RAIN.cmd", "./install.sh", "python rain_lab.py"],
    "docs/setup-guides/README.md": ["INSTALL_RAIN.cmd", "./install.sh", "python rain_lab.py"],
    "docs/setup-guides/one-click-bootstrap.md": ["INSTALL_RAIN.cmd", "./install.sh", "python rain_lab.py"],
    "docs/troubleshooting.md": ["./install.sh", "INSTALL_RAIN.cmd"],
    "docs/ops/troubleshooting.md": ["./install.sh", "INSTALL_RAIN.cmd"],
}

PRIMARY_DOCS = tuple(PRIMARY_DOC_EXPECTATIONS)

LEGACY_DOC_MARKERS = (
    "MultiplicityFoundation/R.A.I.N.",
    "rainlabs.sh/install.sh",
    "R.A.I.N. onboard",
    "./install.sh --install-rust",
    "./install.sh --install-system-deps",
    "./install.sh --prefer-prebuilt",
    "./install.sh --prebuilt-only",
    "./install.sh --force-source-build",
    "./install.sh --docker",
)


def _read(repo_root: Path, rel_path: str) -> str:
    return (repo_root / rel_path).read_text(encoding="utf-8")


def test_primary_docs_keep_current_install_story(repo_root: Path) -> None:
    for rel_path, expected_fragments in PRIMARY_DOC_EXPECTATIONS.items():
        text = _read(repo_root, rel_path)
        for fragment in expected_fragments:
            assert fragment in text, f"{rel_path} is missing required cohesion marker: {fragment!r}"


def test_primary_docs_reject_legacy_repo_and_installer_markers(repo_root: Path) -> None:
    for rel_path in PRIMARY_DOCS:
        text = _read(repo_root, rel_path)
        for marker in LEGACY_DOC_MARKERS:
            assert marker not in text, f"{rel_path} still contains legacy marker: {marker!r}"


def test_clone_instructions_use_current_repo_identity(repo_root: Path) -> None:
    for rel_path in ("README.md", "docs/index.md", "CONTRIBUTING.md"):
        text = _read(repo_root, rel_path)
        assert "https://github.com/topherchris420/james_library.git" in text
        assert "cd james_library" in text


def test_security_doc_uses_current_issue_tracker(repo_root: Path) -> None:
    text = _read(repo_root, "SECURITY.md")
    assert "https://github.com/topherchris420/james_library/issues" in text
    assert "MultiplicityFoundation/R.A.I.N." not in text
