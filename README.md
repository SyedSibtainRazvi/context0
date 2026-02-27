# context0

Save where you left off. Resume in any AI tool — context follows your git branch.

**Open Source** · **Free** · **No Signup** · **No Cloud** · **Local-only** · **MIT License**

<img width="1470" height="802" alt="Screenshot 2026-02-25 at 2 12 27 AM" src="https://github.com/user-attachments/assets/cd17149f-3199-4b66-9f06-fc7f142a1138" />

## Why

AI coding sessions have no memory between tools. Tokens run out in Claude Code, you switch to Cursor, and you're back to re-explaining everything. `context0` fixes that — it saves a checkpoint scoped to your `git repo + branch` so any tool can resume right where you left off.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/SyedSibtainRazvi/context0/main/install.sh | sh
```

Works on macOS (Intel and Apple Silicon) and Linux x86_64.

**Windows:** download the `.zip` from [Releases](https://github.com/SyedSibtainRazvi/context0/releases) and add the binary to your PATH.

## How it works (automatic with MCP)

With MCP configured, you don't run any commands. Just say **"save context"** or **"I'm switching"** and the agent saves everything. On the next session — in any tool — the agent loads it automatically.

**Step 1 — install:**

```bash
curl -fsSL https://raw.githubusercontent.com/SyedSibtainRazvi/context0/main/install.sh | sh
```

**Step 2 — install rule files** (once per project):

```bash
context0 init-rules
```

This tells the AI agent when and how to save/resume context. Writes `CLAUDE.md`, `.cursor/rules/context0.mdc`, and `AGENTS.md` into your project.

**Step 3 — configure MCP** for your tool:

### Claude Code

```bash
claude mcp add context0 context0 mcp-server
```

### Cursor

Cursor doesn't inherit your shell PATH — use the full binary path. Find it with `which context0`, then add to `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "context0": {
      "command": "/Users/your-username/.local/bin/context0",
      "args": ["mcp-server"]
    }
  }
}
```

Restart Cursor after editing.

### Codex

Add to `~/.codex/config.json` using the full path from `which context0`:

```json
{
  "mcpServers": {
    "context0": {
      "command": "/Users/your-username/.local/bin/context0",
      "args": ["mcp-server"]
    }
  }
}
```

**That's it.** The agent handles everything from here. No commands to memorize.

## CLI (optional, no MCP required)

Prefer to do it manually, or don't want MCP setup? The CLI works standalone:

```bash
# Save a checkpoint
context0 save \
  --done "implemented OAuth flow" \
  --next "add refresh token logic" \
  --files src/auth.rs --files src/middleware.rs

# Resume latest checkpoint
context0 resume

# Resume as JSON
context0 resume --json

# Show recent checkpoints
context0 log --limit 20

# Delete checkpoints for current branch
context0 clear
```

## Docs

[https://syedsibtainrazvi.github.io/context0](https://syedsibtainrazvi.github.io/context0)

## How context is scoped

- Key is `git repo root + branch` — `feature/auth` and `main` stay completely separate
- Stored in local SQLite at `~/.context0/context0.db` — nothing leaves your machine
- No cloud, no auth, no runtime dependencies

## Storage

| Setting | Default |
|---|---|
| Database | `~/.context0/context0.db` |
| Override | `context0 --db /path/to/custom.db <command>` |
| SQLite busy timeout | `30000` ms (`CONTEXT0_BUSY_TIMEOUT_MS`) |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
