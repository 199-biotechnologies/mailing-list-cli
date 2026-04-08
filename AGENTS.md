# Agents Guide

`mailing-list-cli` is a single Rust binary for running a mailing list from the
terminal. Campaigns, segments, templates, suppression, analytics, webhook
ingestion — all exposed as JSON-emitting subcommands so an agent can drive
them without an MCP server, schema file, or browser dashboard.

## Current state

- **Version**: v0.2.0 (agent-native rearchitecture, breaking release)
- **Research**: see [/research](./research) for the five dossiers that informed the original design
- **Phase plan**: [docs/plans/2026-04-08-phase-7-v0.2-rearchitecture.md](./docs/plans/2026-04-08-phase-7-v0.2-rearchitecture.md)

## Conventions

This project follows the [agent-cli-framework](https://github.com/199-biotechnologies/agent-cli-framework) patterns:

- Structured JSON output, auto-detected via `IsTerminal`
- Semantic exit codes: `0` success, `1` transient (retry), `2` config (fix setup), `3` bad input, `4` rate limited
- Self-describing via `agent-info` — one command returns the full capability manifest
- **No interactive prompts, ever.** v0.2 removed the v0.1 `template edit` command because it violated this invariant; agents use `Write`/`Edit` tools directly on files passed via `template create --from-file`.
- Local-first state under `~/.local/share/mailing-list-cli/`
- Config under `~/.config/mailing-list-cli/config.toml`
- Cache under `~/.cache/mailing-list-cli/` (always safe to `rm -rf`)
- **Integrated preview.** `template preview <name>` writes rendered HTML to disk and optionally opens it in the default browser. This is the core iteration primitive — it replaces every "catch the mistake upfront" safety net the v0.1 system had.

## Discovery

```bash
mailing-list-cli agent-info
```

Returns a JSON manifest of every subcommand, every flag, every exit code. No documentation drift, no MCP server, no schema file an agent has to load up front.

## Required dependency: email-cli

`mailing-list-cli` does **not** talk to Resend directly. Every send, every audience operation, every event read goes through [`email-cli`](https://github.com/199-biotechnologies/email-cli), which is the sole Resend API client. Both binaries must be on `$PATH`.

This split exists so neither tool has to do the other's job:

- `email-cli` owns the Resend API surface, accounts, profiles, transports, the inbox, the webhook listener.
- `mailing-list-cli` owns campaigns, segmentation, templates, suppression, double opt-in, A/B testing, analytics.

For an agent: use `email-cli` for personal correspondence, `mailing-list-cli` for newsletters and campaigns. They cooperate on the same Resend account but each one stays in its lane.
