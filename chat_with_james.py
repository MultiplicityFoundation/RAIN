import argparse
import io
import os
import sys


JAMES_GREETING = (
    "Systems initialized. I am James, the lead orchestrator for the R.A.I.N. Lab. "
    "Our theoretical sandbox is fully operational. Shall we begin with some speculative physics, "
    "or would you like to review signal theory?"
)
MAX_PAPER_CHARS = 6000


def sanitize_text(text: str) -> str:
    if not text:
        return ""
    for token in ("<|endoftext|>", "<|im_start|>", "<|im_end|>", "|eoc_fim|"):
        text = text.replace(token, "[TOKEN_REMOVED]")
    text = text.replace("###", ">>>")
    text = text.replace("[SEARCH:", "[SEARCH;")
    return text.strip()


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Chat with James in the local research loop.")
    parser.add_argument(
        "--greet",
        action="store_true",
        help="Have James open the session with the theatrical bootstrap greeting.",
    )
    return parser.parse_args(argv)


def _configure_stdout() -> None:
    if hasattr(sys.stdout, "buffer"):
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")


def _load_rlm(repo_root: str):
    sys.path.insert(0, os.path.join(repo_root, "rlm-main", "rlm-main"))
    from rlm import RLM

    return RLM


def _load_james_personality(library_path: str) -> str:
    soul_paths = [
        os.path.join(library_path, "JAMES_SOUL.md"),
        r"james_library\JAMES_SOUL.md",
        r"JAMES_SOUL.md",
    ]

    james_personality = "You are James, a visionary scientist at Vers3Dynamics. You are intense, curious, and precise."
    for path in soul_paths:
        if os.path.exists(path):
            with open(path, "r", encoding="utf-8") as handle:
                james_personality = sanitize_text(handle.read())
            print(f"Soul loaded from: {path}")
            break

    return james_personality


def _build_base_context(james_personality: str) -> str:
    return f"""INTERNAL SYSTEM COMMAND: Activate Persona 'JAMES'.

PROFILE:
{james_personality}

USER: Christopher (Lead Researcher).

INSTRUCTION: Stay in character. You have access to a library. When the user loads a paper, analyze it.
You can write and execute Python code to perform calculations, analysis, or any other task.
When asked to calculate or compute something, write Python code to do so.
"""


def list_papers(library_path: str) -> str:
    if not os.path.exists(library_path):
        return "Library folder not found."
    files = [name for name in os.listdir(library_path) if name.endswith((".md", ".txt"))]
    if not files:
        return "Library is empty."
    return "\n".join([f" - {name}" for name in files])


def read_paper(library_path: str, keyword: str) -> tuple[str | None, str]:
    if not os.path.exists(library_path):
        return None, "Library not found."

    files = [name for name in os.listdir(library_path) if name.endswith((".md", ".txt"))]
    match = next((name for name in files if keyword.lower() in name.lower()), None)

    if match:
        with open(os.path.join(library_path, match), "r", encoding="utf-8") as handle:
            content = handle.read()[:MAX_PAPER_CHARS]
        return match, sanitize_text(content)
    return None, "File not found."


def build_prompt(
    *,
    base_context: str,
    loaded_papers: list[tuple[str, str]],
    conversation_history: list[tuple[str, str]],
    user_message: str,
) -> str:
    prompt_parts = [base_context]

    if loaded_papers:
        prompt_parts.append("\n--- LOADED RESEARCH PAPERS ---")
        for paper_name, paper_content in loaded_papers:
            prompt_parts.append(f"\n[{paper_name}]:\n{paper_content}\n")

    if conversation_history:
        prompt_parts.append("\n--- CONVERSATION HISTORY ---")
        for role, content in conversation_history[-20:]:
            prefix = "Christopher" if role == "user" else "James"
            prompt_parts.append(f"\n{prefix}: {content}")

    prompt_parts.append(f"\nChristopher: {user_message}")
    prompt_parts.append("\nJames:")

    return "\n".join(prompt_parts)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    _configure_stdout()

    repo_root = os.path.dirname(__file__)
    library_path = os.environ.get("JAMES_LIBRARY_PATH", repo_root)
    model_name = os.environ.get("LM_STUDIO_MODEL", "qwen2.5-coder-7b-instruct")
    base_url = os.environ.get("LM_STUDIO_BASE_URL", "http://127.0.0.1:1234/v1")

    RLM = _load_rlm(repo_root)
    james_rlm = RLM(
        backend="openai",
        backend_kwargs={"model_name": model_name, "base_url": base_url},
        environment="local",
        verbose=True,
    )

    james_personality = _load_james_personality(library_path)
    base_context = _build_base_context(james_personality)
    conversation_history: list[tuple[str, str]] = []
    loaded_papers: list[tuple[str, str]] = []

    print("\nJames is listening. (RLM mode with local code execution)")
    print("Commands:")
    print("  /list         Show available research papers")
    print("  /read [name]  Load a paper into James's memory")
    print("  quit          Exit")

    if args.greet:
        print(f"\nJames: {JAMES_GREETING}")
        conversation_history.append(("assistant", JAMES_GREETING))

    while True:
        try:
            user_input = input("\nChristopher: ")
        except (EOFError, KeyboardInterrupt):
            break

        if user_input.lower() in {"quit", "exit"}:
            break

        if user_input.lower() == "/list":
            print(f"\nLibrary contents:\n{list_papers(library_path)}")
            continue

        if user_input.lower().startswith("/read"):
            keyword = user_input.replace("/read", "", 1).strip()
            if not keyword:
                print("Please specify a paper name, for example: /read friction")
                continue

            file_name, content = read_paper(library_path, keyword)
            if file_name is None:
                print(f"Could not find a paper matching '{keyword}'.")
                continue

            print(f"Reading '{file_name}' into memory...", end="", flush=True)
            loaded_papers.append((file_name, content))
            print(" done.")
            user_input = f"I have just loaded the paper '{file_name}'. Please analyze it briefly."

        conversation_history.append(("user", user_input))
        print("James: ", end="", flush=True)

        full_prompt = build_prompt(
            base_context=base_context,
            loaded_papers=loaded_papers,
            conversation_history=conversation_history,
            user_message=user_input,
        )
        result = james_rlm.completion(full_prompt)
        response = result.response if hasattr(result, "response") else str(result)

        print(response)
        conversation_history.append(("assistant", response))

    print("\nJames signing off.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
