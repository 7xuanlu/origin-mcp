# origin-mcp

MCP server for [Origin](https://github.com/7xuanlu/origin). Persistent memory across Claude, ChatGPT, and Cursor.

Origin is a local-first companion for people who work with AI every day. Conversations across tools become connected, deduplicated, and editable. `origin-mcp` is the bridge: it lets any MCP-compatible tool read and write to your shared memory through the [Model Context Protocol](https://modelcontextprotocol.io).

## Install

Add to your MCP config (Claude Code, Cursor, Claude Desktop, Windsurf, Gemini CLI):

```json
{
  "mcpServers": {
    "origin": {
      "command": "npx",
      "args": ["-y", "origin-mcp"]
    }
  }
}
```

Or install the binary directly:

```bash
# Via Homebrew
brew tap 7xuanlu/tap
brew install origin-mcp

# Via cargo
cargo install origin-mcp
```

Then add the binary path to your MCP config:

```json
{
  "mcpServers": {
    "origin": {
      "command": "origin-mcp"
    }
  }
}
```

## How it works

`origin-mcp` connects to the Origin daemon running on `127.0.0.1:7878`. The daemon owns all storage, embeddings, and refinement. This server is a thin MCP interface to it.

```
Claude Code / Cursor / Claude Desktop
    |
    | MCP (stdio)
    v
origin-mcp
    |
    | HTTP
    v
Origin daemon (origin-server)
    |
    v
Local SQLite + embeddings + knowledge graph
```

If the daemon isn't running, `npx origin-mcp` starts it automatically.

## Tools

| Tool | What it does | Annotations |
|------|-------------|-------------|
| `remember` | Store a memory, fact, preference, or decision. The backend auto-classifies type, extracts entities, and links to the knowledge graph. | write, non-destructive |
| `recall` | Search memories and knowledge graph by natural language. Returns ranked results with source tracing. | read-only |
| `context` | Load session context: identity, preferences, goals, and topic-relevant memories. Call this at session start. | read-only |
| `forget` | Delete a specific memory and clean up entity links. Requires the memory ID. | destructive, idempotent |

### What agents should know

The server ships with proactive-capture instructions that guide agents to store the right things at the right granularity. Key ideas:

- **Two mental models**: `profile` (about the user) vs `knowledge` (about the world). Agents should think in these terms when deciding what to store.
- **One idea per memory.** "Prefers TDD" and "uses pytest" are two memories, not one. Specific memories retrieve better than broad summaries.
- **Include the why.** "Switched to dark mode because of migraines" is more useful than "uses dark mode."
- **Omit `memory_type`.** Let the backend auto-classify. Agents get it wrong more often than the classifier.
- **Anti-noise rules.** Don't store conversation filler, tool output, or things trivially re-derivable from code.

See [`src/tools.rs`](src/tools.rs) for the full `with_instructions` text that agents receive.

### Options

```
--origin-url <URL>    Override Origin server URL (default: http://127.0.0.1:7878)
```

## What Origin does with your memories

Origin doesn't just store what agents send. A background engine refines memories over time:

- **Deduplication.** Overlapping memories are merged automatically.
- **Concept distillation.** Related memories are clustered into concepts: compact, wiki-style summaries that save tokens on retrieval.
- **Knowledge graph.** Entities and relations are extracted and linked, so "Alice leads the deploy refactor" connects Alice, the project, and the decision.
- **Contradiction detection.** When new information conflicts with existing memories, Origin surfaces it for your review.

The longer you use it, the better the retrieval gets.

## Requirements

- **Origin daemon** running locally (via the desktop app or `origin-server install`)
- **macOS Apple Silicon** (M1+) at v0.1.0. Linux x64 binaries are built but not yet tested in production.

## License

MIT

## Links

- [Origin](https://github.com/7xuanlu/origin): the desktop app, daemon, and core engine
- [Model Context Protocol](https://modelcontextprotocol.io): the protocol spec
