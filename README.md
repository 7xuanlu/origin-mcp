# origin-mcp

MCP server for [Origin](https://github.com/7xuanlu/origin). Persistent memory for AI work.

Origin is a local-first companion for people who work with AI every day. Chats, projects, and decisions become connected, deduplicated, and editable. `origin-mcp` is the connector: it lets any MCP-compatible tool read and write to your local Origin through the [Model Context Protocol](https://modelcontextprotocol.io).

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

`origin-mcp` connects to the Origin daemon running on `127.0.0.1:7878`. The daemon owns storage, search, embeddings, and refinement. This repo is only the MCP connector.

```
Claude Code / Cursor / Claude Desktop
    |
    | MCP (stdio)
    v
origin-mcp
    |
    | HTTP
    v
Origin runtime
    |
    v
Local SQLite + embeddings + knowledge graph
```

If the daemon is not running, `origin-mcp` returns an actionable setup message. Install the Origin desktop app, or install the headless runtime:

```bash
curl -fsSL https://raw.githubusercontent.com/7xuanlu/origin/main/install.sh | bash
export PATH="$HOME/.origin/bin:$PATH"
origin setup
origin install
origin status
```

## Tools

| Tool | What it does | Annotations |
|------|-------------|-------------|
| `remember` | Store a memory, fact, preference, or decision. The backend auto-classifies type, extracts entities, and links to the knowledge graph. | write, non-destructive |
| `recall` | Search memories and knowledge graph by natural language. Returns ranked results with source tracing. | read-only |
| `context` | Load session context: identity, preferences, goals, and topic-relevant memories. Call this at session start. | read-only |
| `origin_status` | Check daemon reachability, setup mode, Anthropic key state, and local model state. | read-only |
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

Origin works in Basic Memory mode without a local model or API key: storage, search, recall, and MCP memory are available immediately.

When the user opts into a local model or Anthropic key, Origin can refine memories over time:

- **Deduplication.** Overlapping memories are merged automatically.
- **Concept distillation.** Related memories are clustered into concepts: compact, wiki-style summaries that save tokens on retrieval.
- **Knowledge graph.** Entities and relations are extracted and linked, so "Alice leads the deploy refactor" connects Alice, the project, and the decision.
- **Contradiction detection.** When new information conflicts with existing memories, Origin surfaces it for your review.

The longer you use it, the better the retrieval gets.

## Requirements

- **Origin runtime** running locally (via the desktop app or `origin setup` / `origin install`)
- **macOS Apple Silicon** (M1+) at v0.1.0. Linux x64 binaries are built but not yet tested in production.

## License

MIT

## Links

- [Origin](https://github.com/7xuanlu/origin): the desktop app, daemon, and core engine
- [Model Context Protocol](https://modelcontextprotocol.io): the protocol spec
