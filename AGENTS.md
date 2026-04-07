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

## Required dependency: email-cli

`mailing-list-cli` does **not** talk to Resend directly. Every send, every audience operation, every event read goes through [`email-cli`](https://github.com/199-biotechnologies/email-cli), which is the sole Resend API client. Both binaries must be on `$PATH`.

This split exists so neither tool has to do the other's job:

- `email-cli` owns the Resend API surface, accounts, profiles, transports, the inbox, the webhook listener.
- `mailing-list-cli` owns campaigns, segmentation, templates, suppression, double opt-in, A/B testing, analytics.

For an agent: use `email-cli` for personal correspondence, `mailing-list-cli` for newsletters and campaigns. They cooperate on the same Resend account but each one stays in its lane.
