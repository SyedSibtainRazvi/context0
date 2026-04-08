# context0

Ephemeral branch-scoped handoff state for AI coding agents. Like `git stash`, but for your agent's working memory.

**Open Source** · **Free** · **No Signup** · **No Cloud** · **MIT License**

<img width="1470" height="802" alt="context0 in action. Checkpoint saved and resumed in Claude Code." src="https://github.com/user-attachments/assets/cd17149f-3199-4b66-9f06-fc7f142a1138" />

## Why

Every new AI session starts cold. Switch tools, hit a token limit, come back the next day. The agent has no idea what you were doing. `context0` fixes that by saving a compact checkpoint scoped to your git repo and branch, so the next agent resumes exactly where you left off.

It stores the useful handoff state:
- what was done
- what should happen next
- blockers
- test status
- key files
- commit SHA

It does **not** try to be long-term project memory, static project rules, or full conversation replay. You get the state you need to resume work, without dragging along stale assumptions or session junk.

## What context0 is and is not

- `context0` is ephemeral session state for the current branch.
- `context0` is not long-lived project memory like architecture notes or decision logs.
- `context0` is not static instruction files like `CLAUDE.md` or `AGENTS.md`.
- `context0` does not carry over your full chat history. It carries the structured handoff state needed to resume the task cleanly.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/SyedSibtainRazvi/context0/main/install.sh | sh
```

Works on macOS (Intel and Apple Silicon) and Linux (x86_64 and arm64).

**Windows:** download the `.zip` from [Releases](https://github.com/SyedSibtainRazvi/context0/releases) and add the binary to your PATH.

## How it works (automatic with MCP)

With MCP configured, you do not manually run `context0 save` or `context0 resume`. Tell the agent you are switching tools or ending the session and it saves everything through MCP. On the next session, the agent is instructed to load the latest checkpoint for the current branch at session start.

If you prefer, ask Claude or another coding agent to do the setup for you step by step. For example:

```text
Install context0 on this machine, run context0 init-rules in this project, set up MCP for Claude Code, and verify everything step by step.
```

**Step 1. Install:**

```bash
curl -fsSL https://raw.githubusercontent.com/SyedSibtainRazvi/context0/main/install.sh | sh
```

**Step 2. Install rule files** (once per project):

```bash
context0 init-rules
```

This writes agent instruction files (`CLAUDE.md`, `.cursor/rules/context0.mdc`, `AGENTS.md`) into your project. They tell the agent when to save and load context automatically. No manual prompting needed.

**Step 3. Configure MCP for your tool:**

### Claude Code

```bash
claude mcp add context0 context0 mcp-server
```

### Cursor

Cursor does not inherit your shell PATH, so use the full binary path. Find it with `which context0`, then add to `~/.cursor/mcp.json`:

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

Use these commands only if you want to work without MCP, or prefer to save and resume manually from the terminal:

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

- Key is `git repo root + branch`. `feature/auth` and `main` stay completely separate.
- Stored in local SQLite at `~/.context0/context0.db`. Nothing leaves your machine.
- No cloud, no auth, no runtime dependencies.

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
