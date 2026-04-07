# Agents Guide

`mailing-list-cli` is currently in **spec and research phase**. The Rust implementation lands once the design is written and approved.

## Current state

- **Spec**: not yet written (in progress)
- **Implementation**: not started
- **Research**: complete — see [/research](./research) for the five dossiers that inform the design

## Conventions (when the binary ships)

This project follows the [agent-cli-framework](https://github.com/199-biotechnologies/agent-cli-framework) patterns:

- Structured JSON output, auto-detected via `IsTerminal`
- Semantic exit codes: `0` success, `1` transient (retry), `2` config (fix setup), `3` bad input, `4` rate limited
- Self-describing via `agent-info` — one command returns the full capability manifest
- No interactive prompts, ever
- Local-first state under `~/.local/share/mailing-list-cli/`
- Config under `~/.config/mailing-list-cli/config.toml`
- Cache under `~/.cache/mailing-list-cli/` (always safe to `rm -rf`)

## Discovery (when the binary ships)

```bash
mailing-list-cli agent-info
```

That command will return a JSON manifest of every subcommand, every flag, every exit code. No documentation drift, no MCP server, no schema file an agent has to load up front.

## Sister tool

For 1:1 email (send, reply, draft, sync), use [`email-cli`](https://github.com/199-biotechnologies/email-cli). Same author, same conventions, same Resend backend. The two tools coexist on `$PATH` and an agent uses both — `email-cli` for personal correspondence, `mailing-list-cli` for newsletters and campaigns.
