# origin-mcp

MCP server for [Origin](https://github.com/7xuanlu/origin). Where understanding compounds.

Connects AI tools (Claude Code, Cursor, Claude Desktop, ChatGPT, Gemini CLI, Windsurf) to Origin's memories, concepts, decisions, and knowledge graph via the [Model Context Protocol](https://modelcontextprotocol.io).

## Install

```bash
cargo install origin-mcp
```

Or via npm (downloads the binary automatically):

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

## Usage

1. Start the Origin daemon (`origin-server`) or desktop app
2. Configure your AI tool with the MCP config above

The server connects to Origin via HTTP (`127.0.0.1:7878`).

### Options

```
--origin-url <URL>    Override Origin server URL
```

## Tools

| Tool | Description | Annotations |
|------|-------------|-------------|
| `remember` | Store a memory, fact, preference, or decision | write, non-destructive |
| `recall` | Search memories and knowledge graph | read-only |
| `context` | Load session context (identity, preferences, goals, relevant memories) | read-only |
| `forget` | Delete a memory and clean up entity links | destructive, idempotent |

## Agent guidance

The server ships with proactive-capture instructions for agents, including a two-model mental framing (`profile` = about the user; `knowledge` = about the world) and anti-noise rules. Agents should generally omit `memory_type` and let the backend auto-classify. See [`src/tools.rs`](src/tools.rs) for inline tool schemas and `with_instructions` text.

## License

MIT
