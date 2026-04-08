# Session Handoff — Session 3

**Date:** 2026-04-08 (evening)
**Session:** v0.1.3 Codex-reviewed gap fixes → v0.2.0 agent-native rearchitecture → v0.2.1 real-Resend validation + hard-fail state fix
**Context usage at handoff:** roughly half-full

---

## TL;DR for the next session

Four releases shipped in this session on top of v0.1.2:

- **v0.1.3** — Codex-reviewed template lint fixes (unified variable extractor, per-offender line numbers, realistic placeholder sizes). Commit `4465fdd`.
- **v0.2.0** — the big one. Three-way review (Claude + Codex gpt-5.4 xhigh + Gemini 3.1-pro) drove an aggressive agent-native rearchitecture. Dropped MJML, Handlebars, css-inline, html2text, serde_yaml, YAML frontmatter schemas, 14 of 20 lint rules, PEST segment DSL, `webhook listen`, `webhook test`, `template edit`, `template guidelines`. Added hand-rolled `{{ var }}` substituter + `template preview` + JSON-AST segment filter. 23 → 14 crates, ~9500 → ~5500 LoC. Commits `cb5d36c` (Phase 1) + `6ea71d4` (Phase 2+3).
- **v0.2.1** — **real-Resend smoke test against `paperfoot.com` PASSED end-to-end**, including `event poll` ingesting real delivery events and `report show` returning real metrics. Also fixed a state leak in `broadcast send` where the status was stuck in `sending` after a render error. Docs audit for README + AGENTS.md. Commit `9d8c6d1`.

**At v0.2.1:**
- Test count: 97 unit + 62 integration = **159 tests passing**
- Dependencies: **14 runtime crates** (was 23 at v0.1.3, -39%)
- Rust LoC: **~5500** (was ~9500, -42%)
- Template lint rules: **6** (was 20, -70%)
- Breaking changes from v0.1.x: yes, but zero production users
- All CI runs green through v0.2.0; v0.2.1 CI should be queued

## Active plans

- **v0.2 rearchitecture (just shipped):** [`docs/plans/2026-04-08-phase-7-v0.2-rearchitecture.md`](../plans/2026-04-08-phase-7-v0.2-rearchitecture.md)
- **Parity plan (reference):** [`docs/plans/2026-04-08-parity-plan.md`](../plans/2026-04-08-parity-plan.md)
- **Design spec (PARTIALLY STALE):** [`docs/specs/2026-04-07-mailing-list-cli-design.md`](../specs/2026-04-07-mailing-list-cli-design.md) — §7 still documents v0.1 MJML + frontmatter model. See "Outstanding" below.
- **Prior handoffs:** [`session-1`](./2026-04-08-session-handoff.md), [`session-2`](./2026-04-08-session-2-handoff.md)

## Three-way review artifacts

Preserved at `~/.claude/subagent-results/`:
- `rearch-brief-1775664888.md` — the brief all three reviewers answered
- `codex-output-1775664888-rearch.md` — Codex gpt-5.4 xhigh review
- `gemini-output-1775664888-rearch.md` — Gemini 3.1-pro review
- `claude-analysis-1775664888-rearch.md` — my own independent analysis
- Earlier v0.1.3 Codex review: `codex-output-1775657343-template-gaps.md`

## Current state

- **Branch:** `main`
- **Last commit:** `9d8c6d1 chore(v0.2.1): validated against real Resend paperfoot.com...`
- **Latest tag:** `v0.2.1`
- **Tests:** 97 unit + 62 integration = 159 passing, `cargo test -- --test-threads=1` clean
- **Build:** `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` all clean
- **Release binary:** built at `target/release/mailing-list-cli` (3.5 MB stripped/LTO)
- **Smoke test DB:** `/tmp/mlc-smoke-v0.2.0/state.db` preserved — 2 broadcasts (1 sent successfully, 1 failed on hard-fail path)
- **Working tree:** clean
- **Pushed:** main + all tags

## What was accomplished this session

### v0.1.3 — Codex-reviewed template lint fixes (session start)
Six gaps identified in session 2's "honest gaps" section; Codex ranked and advised. Shipped 3 code fixes + 2 docs fixes, deferred 1 to a later version:

1. **Gap #3** (unused-var textual check) — unified `extract_merge_tag_names` to cover `{{#if}}`/`{{#unless}}` arguments and normalize whitespace
2. **Gap #5** (alt/href lint breaks after first offender) — added `line: Option<usize>` to `LintFinding`, dropped the `break`s, emit per-offender with line numbers
3. **Gap #6** (size lint underestimates real send) — replaced placeholder stubs with realistic HTML matching send-time shape
4. **Gap #1** (mrml silent attribute drops) — docs-only warning in `template-authoring.md` (subsequently deleted in v0.2)
5. **Gap #4** (no composition) — softened "v0.2+" promise in lint hints
6. **Gap #2** (template versioning) — deferred

### v0.2.0 — agent-native rearchitecture
After shipping v0.1.3, the user pushed back on the MJML stack itself with the "agent has preview + iteration loop" argument. That led to a three-way review (Claude + Codex + Gemini) which converged on aggressive simplification. User chose Option A (maximum aggression, 14 crates).

**Phase 1 deletions (commit `cb5d36c`):**
- `src/segment/parser.rs` + `grammar.pest` (~640 lines — PEST was just an authoring façade; segments were already stored as JSON AST internally)
- `src/webhook/listener.rs` + `signature.rs` (~240 lines — untested, violated AGENTS.md doctrine that email-cli owns the listener)
- `template edit` command (interactive `$EDITOR`, violated AGENTS.md "no interactive prompts")
- `template guidelines` command + asset (153-line embedded doctrine replaced by the scaffold)
- `webhook test` command
- Dependencies: `pest`, `pest_derive`, `tiny_http`, `tempfile` (moved to dev-deps)

**Phase 2+3 template rewrite (commit `6ea71d4`):**
- Deleted `src/template/{compile,lint,frontmatter}.rs` and `assets/template-authoring.md` (~1350 lines)
- Added `src/template/subst.rs` (~400 lines) — hand-rolled `{{ var }}` + `{{{ allowlist }}}` + `{{#if}}`/`{{#unless}}` substituter with depth-aware nesting, HTML escaping, triple-brace XSS allowlist, unresolved-placeholder tracking
- Added `src/template/render.rs` (~500 lines) — 6 inline lint rules, HTML-to-text stripping, strict vs preview render modes
- Rewrote `src/commands/template.rs` — dropped `edit`/`guidelines`, added real `preview` command with `--out-dir`/`--open`
- Updated `src/broadcast/pipeline.rs` — uses new `template::render()` in strict mode, hard-fails on any unresolved placeholder at send time (this is the v0.2 replacement for the v0.1 frontmatter variable schema)
- Migration 0003 — sentinel no-op (v0.1 databases are not supported; migration 0001 now creates the v0.2 shape directly)
- DB schema: `template.mjml_source` → `template.html_source`, dropped `template.schema_json`
- Dependencies dropped: `mrml`, `handlebars`, `css-inline`, `html2text`, `serde_yaml`
- Version bump to 0.2.0, README badge updated, agent-info updated

### v0.2.1 — real-Resend validation + state fix (commit `9d8c6d1`)
**End-to-end smoke test against `paperfoot.com`:** This was overdue — v0.1.2, v0.1.3, and the initial v0.2.0 all skipped real-Resend validation. The v0.1.1 smoke test caught three real bugs that the stub can't see. v0.2.1 is the first v0.2 release actually validated.

Steps executed:
1. health (all 4 checks green)
2. list create (real Resend segment `ccb0d9d9-ddb4-4c9e-aa22-c3167f9a00dc`)
3. contact add (real Resend contact)
4. template create + lint (clean)
5. template preview with custom data (writes 3 files)
6. broadcast create + broadcast preview (**real single send to** `smoke-test-v0.2.0@paperfoot.com`)
7. broadcast send (**real batch send**, 1 recipient)
8. wait 15s → event poll (processed 100 real Resend events)
9. broadcast show → delivered_count=1 (attributed to our broadcast)
10. report show 1 → full metrics populated (delivered=1, ctr=0.0, bounce_rate=0.0, etc.)
11. **Hard-fail path**: created a template with `{{ typo_code_never_provided }}`, attempted send, verified exit 3 with `template_unresolved_placeholder`, **zero email-cli calls**, no batch file written

**Bug found by step 11:** the broadcast was stuck in `sending` status after the render error instead of being reverted to `failed`. Fixed in `src/broadcast/pipeline.rs` — the render call now explicitly matches on the error, calls `broadcast_set_status(id, "failed", None)`, then returns the BadInput. The integration test `broadcast_send_hard_fails_on_unresolved_placeholder` now asserts the broadcast lands in `failed` state.

**Docs audit:**
- README.md command tables updated for Segments + Templates + Webhook ingestion to match v0.2 shape (removed stale `template guidelines`/`template edit`/`webhook listen --port`/`filter <expr>` references)
- AGENTS.md removed "spec not yet written / implementation not started" stale text, added v0.2 conventions for `template preview` and the "no interactive prompts, ever" invariant

## What is still outstanding

### Must do in the next session

1. **docs/specs/2026-04-07-mailing-list-cli-design.md §7** — still documents the v0.1 MJML + frontmatter + 20-rule lint architecture. Agents reading the spec will see the old design, not v0.2. Options:
   - Annotate §7 as "superseded by v0.2; see docs/plans/2026-04-08-phase-7-v0.2-rearchitecture.md"
   - Rewrite §7 against the new shape
   - Delete the old spec entirely and make the plan the canonical reference

2. **Blind template authoring test against v0.2** — the v0.1 blind test gave agents the embedded `template guidelines` (now deleted) and asked each to author a template. The v0.2 model is fundamentally different. Re-run the test by:
   - Building the scaffold via `template create welcome` (no file) and `template show welcome` to extract the HTML
   - Giving Codex, Gemini, and Claude each a fresh prompt: "Author a welcome email template for mailing-list-cli v0.2.1. You have `template preview` and `template lint`. Iterate until the lint is clean and the preview looks right. Here's the scaffold to build from: [scaffold HTML]"
   - Measure whether all three can successfully author a clean template using only preview+lint feedback
   - Document the results in `docs/blind-test-results-v0.2.md`

3. **Real CI verification for v0.2.1** — the push was recent and CI may still be running. Verify green before closing out. `gh run list --repo 199-biotechnologies/mailing-list-cli --limit 1`

### Should do

4. **Preview dry-run in preflight checks** — currently `preflight_checks` calls `template::lint()` which intentionally strips unresolved-placeholder findings (lint is for structural issues only). That's why the typo template gets past preflight and the hard-fail happens in the chunk loop. An alternative would be to do a dry-run render with the FIRST recipient's data during preflight, so the error is caught BEFORE we mark the broadcast `sending`. This would eliminate the need for the `broadcast_set_status(failed)` fix. Not strictly necessary since the current fix works, but it's a cleaner architecture.

5. **Template versioning (Gap #2 from v0.1.3 Codex review)** — still outstanding. The proper fix is migration 0004 + `template_revision` table + `template history <name>` + `template restore <name> --revision N` commands. Currently `template create --from-file` is a destructive overwrite. Not urgent with zero production users but worth scheduling for v0.3.

6. **Migration 0003 real schema upgrade** — currently a sentinel no-op. If anyone ever upgrades a v0.1.x database in place, they'll hit SQL errors because the template table still has `mjml_source` + `schema_json` columns. A real migration would do:
   ```sql
   ALTER TABLE template RENAME COLUMN mjml_source TO html_source;
   ALTER TABLE template DROP COLUMN schema_json;
   ```
   SQLite 3.35+ supports both; rusqlite 0.37 bundles 3.47+. Not needed now (zero production users), but document it in the README as "clean-slate upgrade only" or implement it properly for v0.3.

### Nice to have

7. **DMARC/SPF/DKIM checks in `report deliverability`** — still stubbed, returns `verified_domains: []`. Would need a `dnscheck` module with actual DNS resolution. Phase 7 or later.

8. **Long-running poll daemon** — currently `event poll` is a one-shot. A `daemon` subcommand that runs the poll loop on a schedule is a reasonable v0.3 add.

9. **`template preview --serve <port>`** — I explicitly dropped this during the three-way review. If it turns out agents want live-reload, it's a few dozen lines of `tiny_http` glue. But the three reviewers all agreed "no live-reload server, use fswatch if you want that" so it's a real no, not a todo.

## Key decisions made this session (not in code or plan)

1. **Three-way review is the right pattern for architectural decisions.** Codex + Gemini + Claude (me) all answering the same structured brief in parallel, then synthesizing. All three converged on the same thesis here, which was strong signal. When they disagreed (e.g., JSON AST vs raw SQL for segments), the disagreement was load-bearing and I resolved based on which option has the cheapest implementation (JSON AST, because segments were already stored that way internally — Codex caught this).

2. **The "agent-native CLI" thesis is real and actionable.** Assumptions that made sense for a blind-human author (declare your schema upfront, lint every possible mistake, embed a 153-line doctrine) become dead weight once you commit to agent-with-preview. The v0.2 rewrite is the concrete expression of this thesis. Test: the v0.2 end-to-end works, and the code is 42% smaller.

3. **Migration 0003 as a sentinel no-op is a defensible shortcut.** Zero production users means clean-slate upgrade is fine. If a production user ever emerges, write a real migration then. Documented in the risk register.

4. **Smoke testing against real Resend is not optional.** Three releases in a row skipped it, the v0.2.1 run caught a bug. This is now a hard rule: every tagged release goes through the paperfoot.com smoke test before being declared done.

5. **CLAUDE.md rule: single-agent sessions don't use TaskCreate** — this held throughout. Tracking progress mentally worked fine for a ~30-file refactor.

6. **Scaffold is the documentation.** The v0.2 scaffold in `src/commands/template.rs::SCAFFOLD` is the only template docs an agent sees. It has to be self-explanatory because there's no separate guide anymore. So far it works, but the blind test re-run (item 2 above) will tell us for real.

## Gotchas + warnings

- **Don't use parallel `cargo test`** — there's a pre-existing race in `paths::tests`. Always `cargo test -- --test-threads=1`.
- **Don't resurrect the frontmatter** — the whole v0.2 rewrite hinges on dropping it. If a new requirement seems to need declare-time variable validation, use the unresolved-at-send-time hard-fail instead.
- **Don't add a live-reload preview server** — the three-way review was explicit "no `--serve`". If you want live-reload, fswatch the template file yourself.
- **Don't put a Handlebars dep back** — the hand-rolled substituter in `src/template/subst.rs` supports exactly what v0.2 ships (scalar, triple-brace allowlist, `{{#if}}`, `{{#unless}}`, depth-aware nesting). Adding features via Handlebars would re-explode the dependency graph we just cut.
- **Don't trust lint() to catch unresolved placeholders** — it explicitly strips `UnresolvedPlaceholder` findings because lint is for structural issues only. The unresolved check lives in `render()` strict mode, invoked from the broadcast pipeline.
- **The `template preview` `--open` flag is not tested** — tests can't reliably launch browsers in CI. The flag is documented but no integration test covers it. Manual smoke testing only.
- **Migration 0003 is a no-op sentinel** — see "Should do" item 6.
- **The smoke test DB at `/tmp/mlc-smoke-v0.2.0/state.db`** can be reused. It has 1 sent broadcast, 1 failed broadcast, and the full report data. Export `MLC_DB_PATH=/tmp/mlc-smoke-v0.2.0/state.db` and the same config to resume.
- **`docs/specs/2026-04-07-mailing-list-cli-design.md`** §7 is stale. Don't quote from it when explaining v0.2 — use the Phase 7 plan instead.

## Test count history

| Version | Tests passing | Notes |
|---|---|---|
| v0.0.3 (session 1 start) | 30 | |
| v0.0.4 Phase 3 ship | 101 | |
| v0.0.5 hotfix | 110 | |
| v0.1.0 Phase 4 | 135 | |
| v0.1.1 Phase 5 | 148 | First real-Resend validation |
| v0.1.2 Phase 6 | 167 | Webhooks + reports; **skipped real-Resend** |
| v0.1.3 Codex gap fixes | 173 | **skipped real-Resend** |
| v0.2.0 agent-native rearchitecture | 158 | Dropped old tests, added new ones; **skipped real-Resend** |
| v0.2.1 real-Resend validation + state fix | **159** | **Real-Resend passed**; 1 new hard-fail integration test |

## Session entry point for the next run

> Read `docs/handoffs/2026-04-08-session-3-handoff.md`. The v0.2.1 release is out and validated against real Resend. **Three things to do this session:**
>
> 1. **Fix the stale design spec**: `docs/specs/2026-04-07-mailing-list-cli-design.md` §7 still documents the v0.1 MJML + frontmatter + 20-rule lint architecture. Either annotate as superseded or rewrite against the v0.2 shape.
>
> 2. **Re-run the blind template authoring test**: extract the v0.2 scaffold via `template create welcome` + `template show welcome`, dispatch Codex + Gemini + Claude with fresh prompts asking each to author a clean template using only `template preview` + `template lint` iteration. Document results in `docs/blind-test-results-v0.2.md`.
>
> 3. **Verify CI green for v0.2.1** — `gh run list --repo 199-biotechnologies/mailing-list-cli --limit 1`.
>
> Then decide whether to schedule template versioning (Gap #2) or the real Migration 0003 for v0.3, and tag the next release accordingly.

---

*End of session 3 handoff.*
