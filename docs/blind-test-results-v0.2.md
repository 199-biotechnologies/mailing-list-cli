# Blind Template Authoring Test — v0.2.2 Results

**Date:** 2026-04-08
**Binary:** `mailing-list-cli 0.2.2` (commit `5bad68a`)
**Participants:** Codex (gpt-5.4, reasoning_effort=xhigh) · Gemini (gemini-3.1-pro-preview auto) · Claude (general-purpose subagent, Opus 4.6)

## Purpose

Validate the v0.2 thesis that **the built-in template scaffold is sufficient cold-start documentation for an AI agent.** If agents cold-started with only the scaffold (plus a short statement of the 6 lint rules and the supported substituter syntax) could author a clean, realistic welcome email via `template preview` + `template lint` iteration, the 153-line `assets/template-authoring.md` and 20-rule lint set deleted in v0.2.0 were correctly identified as dead weight.

## Setup

Each agent ran in an isolated environment under `/tmp/mlc-blind-v020/{agent}/` with its own config, SQLite DB, cache dir, and working directory. Each was given an identical prompt containing:

- The scaffold HTML verbatim (42 lines)
- The 6 lint rules (unsubscribe link, physical address footer, size, forbidden tags, unresolved placeholders, XSS allowlist)
- The supported substituter syntax (`{{ var }}`, `{{{ allowlist }}}`, `{{#if}}`, `{{#unless}}`, depth-aware nesting)
- A task: invent a fictional consumer product and write a polished welcome email using only `template create` / `template lint` / `template preview` iteration.

**Integrity rules:** no reading anything under `src/`, `docs/`, `assets/`, or `tests/`. The scaffold was the only reference.

## Results at a glance

| Agent | Iterations to clean lint | Iterations to acceptable preview | Product invented |
|---|---|---|---|
| **Claude** | 1 | 1 (+1 to verify `{{#unless}}` branch) | **Paperbark** — monthly wild-foraged Australian bush-tea subscription shipped from Daylesford |
| **Gemini** | 1 | 1 | **BrewBot** — AI-powered smart coffee roaster that profiles your palate |
| **Codex** | 1 | 2 | **Larkline Coffee Club** — small-batch cafe-level coffee subscription |

**Verdict: PASS.** All three agents achieved 0 errors / 0 warnings on their **first** `template create` + `template lint` cycle. None of the three needed to iterate on lint errors, and all three produced a realistic preview within 1–2 `template preview` runs. The v0.2 scaffold-as-documentation thesis holds.

## Footgun inventory

Findings from all three reports, deduplicated and prioritized by impact and by how many agents independently flagged the same issue.

### High priority (multi-agent signal)

1. **`template --help` still says "Manage MJML templates (with the embedded agent authoring guide)".** Flagged by both Claude and Gemini as actively misleading — directly contradicts the v0.2.2 reality. Without the prompt's explicit "NOT MJML" warning, both agents said they would have wasted an iteration trying `<mj-section>`. **FIXED in this session** (`src/cli.rs:60`).
2. **`template create --subject` help mentions a "v0.1 transitional path" where subject may be omitted and pulled from frontmatter.** Flagged by Codex as distracting in a blind v0.2.2 test. No such path exists anymore. **FIXED in this session** (`src/cli.rs:339-342`).
3. **`template preview` vs `template render` behavior is inconsistent.** Flagged by Codex: `template preview` auto-injects stub HTML for `{{{ unsubscribe_link }}}` and `{{{ physical_address_footer }}}`, but `template render` leaves them literal unless `--with-placeholders` is passed. Two commands with overlapping purposes and different defaults is a recipe for confusion. **Follow-up needed** — either unify the defaults or document the distinction in both `--help` strings.

### Medium priority (single-agent signal, real bugs)

4. **Named HTML entities are not decoded in the plain-text fallback.** Flagged by Claude. `html_to_text()` in `src/template/render.rs:249` decodes `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#39;`, `&nbsp;` — but not `&mdash;`, `&ndash;`, `&middot;`, `&hellip;`, `&lsquo;`, `&rsquo;`, `&ldquo;`, `&rdquo;`, `&copy;`, `&reg;`, `&trade;`, or numeric character references. If agents write semantic entities for typographic quality the text MIME part renders literal `&mdash;` etc. **Follow-up needed.** Either decode the common named entities + numeric refs, or add a lint rule warning "prefer literal UTF-8 for typographic characters".
5. **`template preview` injects `physical_address_footer` as block HTML (`<div>`) into placements that may be inside a `<p>`.** Flagged by Claude and Codex. The scaffold places both placeholders inside a `<p>` wrapper, but preview's stub is a `<div>` — invalid HTML nesting in the rendered preview. Codex worked around it by changing the wrapper to `<div>`. **Follow-up needed.** Either change the stub to an inline element (`<span>`) or change the scaffold wrapper to `<div>`.

### Low priority (scaffold documentation gaps)

6. **`{{#unless}}` pairing example.** Claude and Codex both guessed at the semantics and got lucky. A one-line comment in the scaffold showing an `{{#if}}` / `{{#unless}}` pair on the same variable would remove the guesswork.
7. **Subject-line merge tags.** Claude noted the scaffold and help don't mention that the `--subject` argument itself can contain `{{ var }}` merge tags. Worth one line in `template create --help`. (Partially addressed by the v0.2.2 `--subject` help rewrite in this session.)
8. **Loop support.** Gemini assumed (correctly) that `{{#each}}` is not supported. The scaffold could state this explicitly to save the guess.
9. **Why triple-brace is limited.** Gemini suggested the scaffold should explain *why* the triple-brace allowlist exists (XSS protection) — agents who've used other template systems may reach for `{{{ }}}` cosmetically.
10. **Whitespace from suppressed `{{#if}}` blocks.** Claude noted a suppressed conditional leaves a blank line in the output. One-line note would help.
11. **Conditional variables and the unresolved-placeholder check.** Codex noted it had to empirically test whether a variable used only inside `{{#if name}}...{{/if}}` but absent from the merge data would be flagged as unresolved. (Answer: no, it's treated as falsy and the block is suppressed.) Worth documenting.
12. **Preview artifact filenames.** Codex noted `template preview --help` doesn't specify that the output directory will contain `index.html`, `plain.txt`, and `subject.txt`. One line in the `--help` string would help.

### Prompt-side issue (my fault, not a code issue)

13. **My dispatch note lied about preview injection.** I wrote "preview mode doesn't inject them — that is expected". In reality, `template preview` DOES inject stubs. Both Claude and Codex caught the contradiction. This is a bug in the blind-test prompt, not the code, but it reinforces finding #3.

## Creative output quality

All three agents produced plausible product copy. No two invented the same product (tea, AI roaster, coffee subscription). All three correctly used `{{#if}}` conditionals for optional merge fields (Claude: `referral_code`; Codex: first-order discount; Gemini: discount code). None tried to use MJML tags, named helpers, or unsupported syntax. None tried to read forbidden files.

**The integrity of the test held:** the scaffold + 6-rule list + supported-syntax list was enough to produce clean, compliant templates with no detours.

## Actions taken in this session

- Fixed finding #1 (`template --help` MJML string) in `src/cli.rs`.
- Fixed finding #2 (`template create --subject` v0.1 frontmatter text) in `src/cli.rs`.
- Wrote this summary document.

## Follow-ups proposed (not yet shipped)

| # | Finding | Scope | Ship where |
|---|---|---|---|
| 3 | `render` vs `preview` default inconsistency | S | v0.2.3 or v0.3 |
| 4 | Named HTML entity decode in plain-text fallback | S | v0.2.3 or v0.3 |
| 5 | `physical_address_footer` stub block-level nesting | XS | v0.2.3 |
| 6–12 | Scaffold + `--help` documentation micro-improvements | XS each | v0.2.3 rollup |

**Recommendation:** roll findings 3–12 into a `v0.2.3` patch release before tackling v0.3's larger scope items (template versioning, DMARC, daemon). All are small, well-understood, and fix real agent-facing friction that real users will hit within the first five minutes of using the CLI. v0.2.3 should also go through the mandatory paperfoot.com smoke test.

## Raw artifacts

- **Claude report:** `~/.claude/subagent-results/blind-test-v020-claude-1775687631.md`
- **Gemini report:** `~/.claude/subagent-results/blind-test-v020-gemini-1775687631.md`
- **Codex report:** `~/.claude/subagent-results/blind-test-v020-codex-1775687631.md` (sandbox initially blocked the write; copied from `/tmp/mlc-blind-v020/codex/work/`)
- **Per-agent working dirs preserved:** `/tmp/mlc-blind-v020/{claude,gemini,codex}/work/`
- **Dispatch prompt:** `/tmp/mlc-blind-v020/prompt.md`
