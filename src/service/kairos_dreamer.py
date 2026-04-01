import os
import json
import asyncio
from pydantic import BaseModel, Field
from typing import List
from langchain_openai import ChatOpenAI
from langchain_core.prompts import ChatPromptTemplate

IS_WINDOWS = os.name == 'nt'
SOCKET_PATH = "/tmp/kairos_dreamer.sock"

class KnowledgeNode(BaseModel):
    entity: str = Field(description="The core subject")
    relationship: str = Field(description="How it relates")
    target: str = Field(description="The object of the relationship")
    context: str = Field(description="Dense summary of the memory")

class KairosConsolidation(BaseModel):
    compressed_nodes: List[KnowledgeNode]

async def process_batch(memories: List[dict]) -> str:
    if not memories: return json.dumps({"compressed_nodes": []})

    llm = ChatOpenAI(model="gpt-4o-mini", temperature=0.1) 
    prompt = ChatPromptTemplate.from_messages([
        ("system", "Extract factual assertions, preferences, and states into dense knowledge nodes. Ignore pleasantries."),
        ("human", "Raw logs:\n{raw_logs}")
    ])
    chain = prompt | llm.with_structured_output(KairosConsolidation)
    
    script = "\n".join([f"[{m.get('timestamp', '')}] {m.get('role', 'user')}: {m.get('content', '')}" for m in memories])

    try:
        result = await chain.ainvoke({"raw_logs": script})
        return result.model_dump_json()
    except Exception as e:
        print(f"[KAIROS ERROR] {e}")
        return json.dumps({"compressed_nodes": []})

async def handle_client(reader, writer):
    data = await reader.read()
    if data:
        payload = json.loads(data.decode('utf-8'))
        compressed = await process_batch(payload.get("memories", []))
        writer.write(compressed.encode('utf-8'))
        await writer.drain()
    writer.close()

async def main():
    if IS_WINDOWS:
        server = await asyncio.start_server(handle_client, '127.0.0.1', 50051)
        print("[KAIROS] Listening on TCP 50051 (Windows Fallback)")
    else:
        if os.path.exists(SOCKET_PATH): os.remove(SOCKET_PATH)
        server = await asyncio.start_unix_server(handle_client, path=SOCKET_PATH)
        print(f"[KAIROS] Listening on UDS {SOCKET_PATH}")

    async with server:
        await server.serve_forever()

if __name__ == "__main__":
    asyncio.run(main())
