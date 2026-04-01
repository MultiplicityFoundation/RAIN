import os
import sys
import json
import asyncio
from typing import List, Dict, Any
from pydantic import BaseModel, Field
from langchain_openai import ChatOpenAI
from langchain_core.prompts import ChatPromptTemplate

# Configuration based on the existing Rust implementation
IS_WINDOWS = os.name == "nt"
SOCKET_PATH = "kairos.sock"
TCP_PORT = 48765


# The Rust daemon expects a DreamFact, which maps perfectly to the requested KnowledgeNode
class KnowledgeNode(BaseModel):
    entity: str = Field(description="The core subject")
    relationship: str = Field(description="How it relates")
    target: str = Field(description="The object of the relationship")
    context: str = Field(description="Dense summary of the memory")


class KairosConsolidation(BaseModel):
    compressed_nodes: List[KnowledgeNode]


async def process_batch(rows: List[Dict[str, Any]]) -> str:
    """Processes a batch of memory rows from SQLite through the LLM."""
    if not rows:
        return json.dumps({"source_ids": [], "facts": []})

    llm = ChatOpenAI(model="gpt-4o-mini", temperature=0.1)
    prompt = ChatPromptTemplate.from_messages(
        [
            (
                "system",
                "Extract factual assertions, preferences, and states into dense knowledge nodes. Ignore pleasantries.",
            ),
            ("human", "Raw logs:\n{raw_logs}"),
        ]
    )
    chain = prompt | llm.with_structured_output(KairosConsolidation)

    # Format the raw logs into a script for the LLM
    script_lines = []
    for m in rows:
        timestamp = m.get("timestamp", "")
        role = m.get("role", "user")
        content = m.get("content", "")
        script_lines.append(f"[{timestamp}] {role}: {content}")

    script = "\n".join(script_lines)
    source_ids = [m.get("id") for m in rows if "id" in m]

    try:
        # Await the structured output
        result = await chain.ainvoke({"raw_logs": script})

        # Map the Python pydantic structure (KnowledgeNode) back to the Rust expected shape (DreamFact)
        facts = [
            {"entity": node.entity, "relationship": node.relationship, "target": node.target, "context": node.context}
            for node in result.compressed_nodes
        ]

        # Build the exact JSON shape expected by Rust: KairosBatchResponse
        resp = {"source_ids": source_ids, "facts": facts}
        return json.dumps(resp)

    except Exception as e:
        print(f"[KAIROS ERROR] {e}", file=sys.stderr)
        return json.dumps({"source_ids": source_ids, "facts": []})


async def handle_client(reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
    """Handles an IPC connection from the Rust daemon."""
    try:
        # The Rust side writes JSON and ending with '\n' (exchange_json_stream)
        data = await reader.readline()
        if data:
            payload = json.loads(data.decode("utf-8").strip())

            # The Rust KairosBatchRequest struct has 'request_id' and 'rows'
            rows = payload.get("rows", [])
            print(f"[KAIROS] Processing batch of {len(rows)} memories...")

            compressed = await process_batch(rows)

            # Send back the JSON response followed by a newline, matching what Rust's reader_line expects
            writer.write(compressed.encode("utf-8"))
            writer.write(b"\n")
            await writer.drain()
            print(f"[KAIROS] Successfully responded to Rust daemon.")
    except asyncio.IncompleteReadError:
        pass
    except Exception as e:
        print(f"[KAIROS DECODE ERROR] {e}", file=sys.stderr)
        # Fallback empty response to unblock the Rust daemon
        writer.write(b'{"source_ids": [], "facts": []}\n')
        await writer.drain()
    finally:
        writer.close()
        await writer.wait_closed()


async def main():
    if IS_WINDOWS:
        server = await asyncio.start_server(handle_client, "127.0.0.1", TCP_PORT)
        print(f"[KAIROS] Dreaming Daemon Worker Listening on TCP {TCP_PORT} (Windows)")
    else:
        # Cleanup stale socket file
        if os.path.exists(SOCKET_PATH):
            os.remove(SOCKET_PATH)
        server = await asyncio.start_unix_server(handle_client, path=SOCKET_PATH)
        print(f"[KAIROS] Dreaming Daemon Worker Listening on UDS {SOCKET_PATH}")

    async with server:
        await server.serve_forever()


if __name__ == "__main__":
    asyncio.run(main())
