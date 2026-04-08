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

Works on macOS (Intel and Apple Silicon) and Linux (x86_64 and arm64).

**Windows:** download the `.zip` from [Releases](https://github.com/SyedSibtainRazvi/context0/releases) and add the binary to your PATH.

## How it works (automatic with MCP)

With MCP configured, you do not manually run `context0 save` or `context0 resume`. Just talk to Claude Code, Cursor, or Codex normally — for example, say you're switching tools or ending the session — and the agent saves everything through MCP. On the next session, the agent loads it automatically.

If you prefer, ask Claude or another coding agent to do the setup for you step by step. For example:

```text
Install context0 on this machine, run context0 init-rules in this project, set up MCP for Claude Code, and verify everything step by step.
```

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

**That's it.** After MCP is configured, you can talk to the agent normally. You do not need to manually run `context0` commands inside Claude Code, Cursor, or Codex.

## Manual CLI (optional, no MCP required)

Use these commands only if you want to work without MCP, or prefer to save/resume manually from the terminal:

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
