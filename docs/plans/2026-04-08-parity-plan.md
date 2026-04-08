# mailing-list-cli — Parity Plan to Beehiiv / MailChimp

**Date:** 2026-04-08
**Status:** Planning document
**Current release:** `v0.0.3` — foundations + lists + contacts (`list create/ls/show`, `contact add/ls`)
**Goal:** ship a mailing-list CLI with the 80/20 feature set that real creators on Beehiiv, Buttondown, MailChimp, MailerLite, and Kit actually use day-to-day, all on top of `email-cli >= 0.6.0`.

---

## 1. What we have today (v0.0.3)

| Shipped | Form |
|---|---|
| Single Rust binary, JSON envelope, semantic exit codes, `agent-info`, `health` | v0.0.1 |
| SQLite with the full 16-table schema (lists, contacts, tags, fields, segments, templates, broadcasts, recipients, suppression, events, clicks, optin tokens, soft-bounce counts) | v0.0.1 |
| Five user-facing commands: `list create`, `list ls`, `list show`, `contact add`, `contact ls` | v0.0.2 |
| Migrated to `email-cli v0.6` (audiences → segments, flat contacts, `--properties` plumbing ready) | v0.0.3 |

**30 tests green. Clippy clean. CI green. Tagged v0.0.3 pushed.**

The whole schema is already in place in SQLite — every missing feature below has a table waiting for it. What's missing is the CLI surface, the business logic, and the wiring.

---

## 2. The 80/20 feature set every real operator uses

Cross-referenced from the two competitor research dossiers in [`/research`](../../research/):

**From creator platforms** (Beehiiv, Buttondown, Substack):

1. **Compose + send a campaign** — subject, from-name, body, scheduled time, markdown or HTML.
2. **Subscriber CRUD + bulk import** — CSV, with tags/source.
3. **Tags + segments** — signup source, tag, behavior (opened/clicked X in last N days).
4. **Send to a segment** — not just "everyone".
5. **Subject-line A/B test** — 2–4 variants, time-bounded, auto-pick winner.
6. **Per-campaign analytics** — opens, clicks, click-per-link, unsubscribes, bounces.
7. **Bounce + complaint handling** — auto-suppress hard bounces, track soft-bounce trend, suppression list export.
8. **Welcome automation / drip** — 2-step sequence triggered by subscribe-event.

**From marketing platforms** (MailChimp, MailerLite, Kit):

9. **Contacts with status enum** — active / unsubscribed / bounced / complained / cleaned.
10. **Typed custom fields** — text / number / date / bool / select.
11. **Tags** — n:m with contacts, manual or automation-applied.
12. **Lists or single-list-with-groups** — stable grouping primitive.
13. **Dynamic segments** — boolean rules over fields, tags, engagement, dates (ALL / ANY / NONE modes).
14. **Forms + double opt-in** — capture + confirmation.
15. **Broadcast / campaign sender** — subject, from, body, recipient filter, schedule.
16. **Automations** — DAG: trigger → wait → condition → action.
17. **Templates + merge tags** — block editor OR HTML + variable substitution.
18. **A/B testing** — subject and content variants, auto-pick by open or click rate.
19. **Bounce/complaint auto-suppression** — hard-bounces cleaned immediately, complaints permanent.
20. **RFC 8058 one-click `List-Unsubscribe`** — mandatory since the Feb/Jun 2024 Gmail/Yahoo enforcement.
21. **Reports** — per-campaign and per-link.
22. **REST API + webhooks** — for programmatic control.

**Deduplicated parity list: ~18 features.** `mailing-list-cli` has 5 of them shipped and 13 still to go.

---

## 3. Your original asks, mapped to features

You explicitly asked for five things when you started this project. Here's where each lands:

| Your ask | Feature it becomes | Ships in |
|---|---|---|
| **Bounce rate tracking** | Aggregate `email.bounced` events per broadcast → `report show <id>` shows `bounced_count` + `bounce_rate` | **Phase 6** (reports & webhooks) |
| **Unsubscribe count per batch** | Each broadcast tracks its own recipient log; unsubscribe events attribute to the originating broadcast_id | **Phase 6** (reports & webhooks) |
| **Segment batches and give them names** | A `broadcast` IS a named, saved campaign targeted at a list or segment. `broadcast create --name "Q4 newsletter" --template x --to segment:engaged` | **Phase 5** (broadcasts) |
| **Click tracking inside email links** | Resend already does link rewriting when the domain has click tracking enabled; `email.clicked` events carry the original URL; `report links <broadcast-id>` aggregates per-link counts | **Phase 6** (reports) |
| **Templates with explicit agent authoring guidelines** | `template create` with MJML source + YAML frontmatter schema; `template guidelines` prints the embedded authoring guide for LLMs to read before authoring | **Phase 4** (templates) |

**Everything you asked for lands across Phases 4–6 (versions v0.1.0 → v0.1.2).** Phase 3 (segments) is a prerequisite for the "send to a named segment" flow. Phases 7–9 are compliance, A/B, and release polish — valuable but not strictly required for your original asks.

---

## 4. Feature matrix — mailing-list-cli vs competitors

Legend: ✅ shipped · 🟡 partially spec'd · 🔴 not started

| Feature | Beehiiv | MailChimp | MailerLite | Kit | mailing-list-cli v0.0.3 | Planned in |
|---|---|---|---|---|---|---|
| Single CLI binary | — | — | — | — | ✅ | v0.0.1 |
| JSON-native output | — | partial | partial | partial | ✅ | v0.0.1 |
| Agent self-description | — | — | — | — | ✅ | v0.0.1 |
| Lists | ✅ | ✅ | groups | tags | ✅ | v0.0.2 |
| Contact CRUD (basic) | ✅ | ✅ | ✅ | ✅ | ✅ (add/ls only) | v0.0.2 |
| Contact CRUD (show/erase/resubscribe) | ✅ | ✅ | ✅ | ✅ | 🔴 | Phase 3 (v0.0.4) |
| Bulk CSV import | ✅ | ✅ | ✅ | ✅ | 🔴 | Phase 3 (v0.0.4) |
| Tags (n:m with contacts) | ✅ | ✅ | ✅ | ✅ | schema only | Phase 3 (v0.0.4) |
| Typed custom fields | ✅ | ✅ | ✅ | ✅ | schema only | Phase 3 (v0.0.4) |
| Dynamic segments (filter language) | ✅ | ✅ | ✅ | ✅ | schema only | Phase 3 (v0.0.4) |
| **Templates (MJML + merge tags)** | block editor | block + HTML | block + HTML | limited | 🔴 | **Phase 4 (v0.1.0)** |
| **Template authoring guide for agents** | — | — | — | — | 🔴 | **Phase 4 (v0.1.0)** |
| Template lint | partial | partial | partial | — | 🔴 | Phase 4 (v0.1.0) |
| **Broadcasts (named campaigns)** | ✅ | ✅ | ✅ | ✅ | 🔴 | **Phase 5 (v0.1.1)** |
| Broadcast preview | ✅ | ✅ | ✅ | ✅ | 🔴 | Phase 5 (v0.1.1) |
| Broadcast schedule / cancel | ✅ | ✅ | ✅ | ✅ | 🔴 | Phase 5 (v0.1.1) |
| Per-recipient RFC 8058 unsubscribe | ✅ | ✅ | ✅ | ✅ | 🔴 (gets it free once we use `email-cli broadcast create`) | Phase 5 (v0.1.1) |
| Physical address footer injection | ✅ | ✅ | ✅ | ✅ | 🔴 | Phase 5 (v0.1.1) |
| **Webhook event ingestion** | — (UI-only) | — | — | — | 🔴 | **Phase 6 (v0.1.2)** |
| **Per-broadcast report (opens/clicks/bounces/unsubs/CTR)** | ✅ | ✅ | ✅ | ✅ | 🔴 | **Phase 6 (v0.1.2)** |
| **Per-link click report** | ✅ | ✅ | ✅ | ✅ | 🔴 | **Phase 6 (v0.1.2)** |
| Engagement / deliverability report | ✅ | ✅ | partial | partial | 🔴 | Phase 6 (v0.1.2) |
| Auto hard-bounce suppression | ✅ | ✅ | ✅ | ✅ | 🔴 | Phase 6 (v0.1.2) |
| Soft-bounce streak auto-suppression | ✅ | ✅ | ✅ | ✅ | 🔴 | Phase 6 (v0.1.2) |
| Global suppression list | ✅ | ✅ | ✅ | ✅ | schema only | Phase 7 (v0.1.3) |
| Double opt-in | ✅ | ✅ | ✅ | ✅ | schema only | Phase 7 (v0.1.3) |
| GDPR contact erase | ✅ | ✅ | ✅ | ✅ | 🔴 | Phase 7 (v0.1.3) |
| Domain auth check (SPF/DKIM/DMARC) | ✅ | ✅ | ✅ | ✅ | 🔴 | Phase 7 (v0.1.3) |
| A/B subject / content test | ✅ | ✅ | ✅ | creator+ | 🔴 | Phase 8 (v0.1.4) |
| Welcome / drip automation | ✅ | ✅ | ✅ | ✅ | 🔴 deferred | Phase 10+ (wraps Resend Automations when it GAs) |
| Daemon / scheduled tick | n/a (SaaS) | n/a | n/a | n/a | 🔴 | Phase 8 (v0.1.4) |
| Homebrew / prebuilt binaries | n/a | n/a | n/a | n/a | 🔴 | Phase 9 (v0.1.5) |

**Total shipped: 6/30 features.** **Target for v0.1.x: 24/30.** Remaining 6 are automations (deferred until Resend's Automations API exits private alpha) and pure release-polish items.

---

## 5. Phased roadmap to parity

Each phase produces a shippable release tag. Order is deliberate: later phases depend on earlier ones.

### Phase 3 — Contacts, tags, fields, segments (v0.0.4)

**Goal:** everything needed to define WHO a campaign is going to, before we can actually send a campaign.

**Ships:**
- `contact show <email>`, `contact erase <email>`, `contact resubscribe <email>`
- `contact import <file.csv> --list <id> [--double-opt-in]` with rate-limit-aware chunking
- `contact tag <email> <tag>` / `contact untag <email> <tag>`
- `contact set <email> <field> <value>` (writes local `contact_field_value`; passes through `--properties` to `email-cli contact update` if a field is marked `sync-to-resend`)
- `tag ls`, `tag rm`
- `field create <key> --type <text|number|date|bool|select>`, `field ls`, `field rm`
- `segment create <name> --filter <expr>`, `segment ls`, `segment show`, `segment members`
- Filter expression parser (`pest` grammar) for `tag:vip AND opened_last:30d AND NOT bounced`-style expressions
- `contact ls --filter <expr>` uses the same parser

**Acceptance criteria:**
- Import a 10,000-row CSV under the 5 req/sec Resend rate limit, resumable on failure.
- `segment members <name>` returns the correct set after a contact gains/loses a matching tag.
- Filter expression parser handles boolean AND/OR/NOT, grouping with parens, and all atom types from the spec §6.

**Why first:** every later phase needs to resolve "who gets this send" and the answer is a filter expression over contacts/tags/fields.

---

### Phase 4 — Templates + the agent authoring guide (v0.1.0)

**Goal:** the template surface — and the thing you specifically asked for: explicit guidelines so agents write templates that actually render.

**Ships:**
- Dependency: [`mrml`](https://crates.io/crates/mrml) (MJML compiler in pure Rust), [`handlebars`](https://crates.io/crates/handlebars) (merge tags), [`css-inline`](https://crates.io/crates/css-inline), [`html2text`](https://crates.io/crates/html2text).
- `template create <name> [--from-file <path>]`
- `template ls`, `template show <name>`
- `template render <name> [--with-data <file.json>]` — prints the final HTML + plain-text
- `template lint <name>` — MJML parse, schema check, flexbox/grid ban, Gmail 102 KB clipping check, required placeholder check
- `template edit <name>` — opens in `$EDITOR`
- `template rm <name> --confirm`
- **`template guidelines`** — prints the embedded agent authoring guide (written as `assets/template-authoring.md`, compiled into the binary via `include_str!`). The guide is the one-pager every LLM reads before authoring its first template. Full content already drafted in spec §16.

**Acceptance criteria:**
- An LLM (Claude, GPT, Gemini) given only the output of `template guidelines` can author a valid MJML template that `template lint` accepts on the first try.
- `template render <name> --with-data {"first_name":"Alice"}` produces HTML that renders correctly in Gmail, Apple Mail, and Outlook desktop (verified manually once via Litmus or equivalent).
- The `{{{ unsubscribe_link }}}` and `{{{ physical_address_footer }}}` placeholders are required and enforced by the linter.

**Why second:** templates are the content Broadcasts send. Can't ship a broadcast without a template.

---

### Phase 5 — Broadcasts + the send pipeline (v0.1.1)

**Goal:** named, segmented, targeted campaigns. This is the big one — the core "mailing list" surface.

**Ships:**
- `broadcast create --name <n> --template <tpl> --to <list-or-segment>` — stages a campaign in `draft` status
- `broadcast preview <id> --to <test-email>` — calls `email-cli send` with a single rendered copy
- `broadcast schedule <id> --at <iso|natural>` — moves from `draft` to `scheduled`
- `broadcast send <id>` — the send pipeline (spec §5)
- `broadcast cancel <id>` — cancels a scheduled broadcast
- `broadcast ls [--status <s>]`, `broadcast show <id>`
- **Send pipeline** (spec §5, fully implemented):
  1. Resolve segment/list → recipient IDs
  2. Pre-flight invariant checks (domain auth, complaint rate, physical address)
  3. Suppression filter (global)
  4. Per-recipient template render (via `mrml` + `handlebars`)
  5. Physical address footer injection (mandatory)
  6. RFC 8058 `List-Unsubscribe` header injection (mandatory, falls back to local signing if not using native broadcasts)
  7. Shell out to `email-cli broadcast create --send` for the preferred path, or `email-cli batch send --file` as fallback
  8. Update `broadcast_recipient` rows from the returned email IDs
  9. Mark `broadcast.status = 'sent'`

**Acceptance criteria:**
- A 10,000-recipient broadcast completes without exceeding the 5 req/sec rate limit.
- Every recipient either receives the email OR is logged in `broadcast_recipient` as `suppressed` with a reason.
- `broadcast cancel` works on `scheduled` state.
- `broadcast show <id>` returns the campaign's current state as JSON.

**Why third:** everything above this phase ("was it sent", "to how many", "what was in it") presumes a broadcast exists.

---

### Phase 6 — Webhook ingestion + reports (v0.1.2)

**Goal:** the metrics surface. This is where your original asks (bounce rate, unsubscribe count, click tracking) become real.

**Ships:**
- `webhook listen [--port 8081]` — local HTTP server that receives Resend webhooks, verifies Svix HMAC signatures, and mirrors events into the local `event` and `click` tables
- `event poll` — alternative path that uses `email-cli email list --after <id>` polling (per spec §13.5 — thanks to the email-cli v0.6 `email list` command)
- Event handling for the 11 Resend email event types (spec §10.2):
  - `email.delivered` → `broadcast.delivered_count++`, `broadcast_recipient.status = 'delivered'`
  - `email.bounced` (type=Permanent) → auto-suppress, `broadcast_recipient.status = 'bounced'`
  - `email.delivery_delayed` → increment soft-bounce streak, auto-suppress at 5 consecutive
  - `email.complained` → auto-suppress with reason `complained`
  - `email.opened` → `click` and `event` rows, `broadcast.opened_count++`
  - `email.clicked` → ditto, plus `click.link` for per-link aggregation
  - `email.suppressed`, `email.failed` → recorded
- **`report show <broadcast-id>`** — opens, clicks, bounces, unsubscribes, complaints, CTR, suppressed count, time series
- **`report links <broadcast-id>`** — click count per link (the explicit user ask)
- `report engagement [--list <id>|--segment <name>]` — engagement scores
- `report deliverability` — domain health (bounce rate, complaint rate, DMARC pass rate)
- A daily sweep that pulls soft-bounce-counter updates and writes engagement scores

**Acceptance criteria:**
- After running a test broadcast to a known set of recipients, `report show <id>` reflects the correct deliver/bounce counts within 60 seconds of the webhook arriving.
- `report links` produces a sorted list of URLs by click count.
- Hard bounces auto-create a `suppression` row with reason `hard_bounced`.

**Why fourth:** this is the payoff phase. Everything the user sees after hitting "send" is produced here.

---

### Phase 7 — Compliance + opt-in + suppression (v0.1.3)

**Goal:** safe operation at scale. Compliance features that protect the sender domain's reputation.

**Ships:**
- `optin start <email> --list <id>` — sends a double opt-in confirmation via `email-cli send` with the embedded confirmation template
- `optin verify <token>` — marks the contact `active` and records `confirmed_at`, `consent_ip`, `consent_user_agent`
- `optin pending` — lists pending opt-ins with token expiry
- `unsubscribe <email>` — honors an unsubscribe, adds to global suppression
- `suppression ls`, `suppression add`, `suppression rm`, `suppression import`, `suppression export`
- `dnscheck <domain>` — verifies SPF / DKIM / DMARC / FCrDNS
- `contact erase <email> --confirm` — GDPR Article 17 hard erasure (spec §9.2 flow)
- Per-jurisdiction consent tracking (CAN-SPAM, GDPR, CASL, PECR)

**Acceptance criteria:**
- `contact erase` removes all PII but keeps the `contact.id` for referential integrity; suppression entry stays forever.
- `dnscheck` refuses to pass if DMARC alignment is broken.
- `optin verify` with an expired token returns exit 3.

**Why fifth:** these are the things that keep the list from getting the domain blacklisted. Important but not part of the user's explicit asks.

---

### Phase 8 — A/B testing + daemon (v0.1.4)

**Goal:** subject + content A/B tests, plus a long-running daemon for scheduled sends and cron tasks.

**Ships:**
- `broadcast ab <id> --vary <subject|body> --variants 2 --sample-pct 10 --winner-by <opens|clicks> --decide-after <duration>`
- `broadcast ab-promote <id>` — manual winner promotion if `winner-by manual`
- Daemon wrapper (`daemon start/stop/status`) running:
  - Scheduled broadcast tick (sends when `scheduled_at` arrives)
  - Soft-bounce sweep
  - Sunset evaluator (re-engagement campaigns for inactive subs)
  - Webhook listener (integrated)

**Acceptance criteria:**
- An A/B test with 10% sample and 4-hour window picks the winning subject and sends it to the remaining 90%.
- `daemon start` survives a kill -HUP (reload config) and kill -TERM (graceful shutdown).

---

### Phase 9 — Release polish (v0.1.5)

**Goal:** distribution + contract testing.

**Ships:**
- Homebrew tap (`199-biotechnologies/homebrew-tap`)
- Prebuilt binaries for macOS (arm64 / x86_64) and Linux on GitHub Releases
- Shell completions (`completions bash|zsh|fish|powershell`)
- Full agent-info contract test (every command listed in `agent-info` must be routable)
- End-to-end fixture tests with the updated stub `email-cli`

**Acceptance criteria:**
- `brew install 199-biotechnologies/tap/mailing-list-cli` works on a clean mac.
- `cargo install mailing-list-cli` works.
- The GitHub release contains signed binaries for all three targets.

---

### Phase 10+ — Automations (deferred)

**When:** once Resend's Automations API exits private alpha.

**What:** mailing-list-cli will wrap `POST /automations` directly via a new `email-cli automation` noun (file ask with the email-cli team). mailing-list-cli's automation surface becomes a thin wrapper instead of a custom DAG engine.

**This is explicitly deferred.** Don't plan around it for now.

---

## 6. Scope discipline rules

These keep the v0.1 cycle from exploding into a multi-year project:

1. **No custom UI.** The terminal is the UI. No web dashboard. No browser-based preview.
2. **No hosted preference center.** Use Resend's hosted page via `email-cli` → Topics (once needed).
3. **No built-in CRM.** Custom fields are the extent. No deal/opportunity/pipeline abstractions.
4. **No multi-channel.** Email only. SMS / push / in-app are explicitly out.
5. **No predictive analytics.** No RFM scoring, no send-time optimization, no churn prediction.
6. **No automations until Resend's API is GA.** We wrap, we don't build.
7. **No per-segment unsubscribe.** The unsubscribe flag is account-wide, as Resend models it. Topics provide per-topic granularity if needed later.
8. **Reports are text + JSON.** No charts, no PDFs, no dashboards. `--json` makes them plottable by any downstream tool the agent wants.

Every feature request that violates one of these rules gets filed as "out of scope for v0.1, revisit after v1.0 if there's demand."

---

## 7. How to execute

Each phase has its own bite-sized implementation plan, written just-in-time when that phase begins, following the same template as [`docs/plans/2026-04-07-phase-1-foundations.md`](./2026-04-07-phase-1-foundations.md):

- Each task is 2–5 minutes of work
- TDD: failing test → minimal implementation → passing test → commit
- Exact file paths, complete code in every step, no placeholders
- Self-review before marking the phase complete

Phases are not parallelizable against each other — they have hard dependencies. But individual tasks within a phase CAN be split across parallel subagents via the [`superpowers:subagent-driven-development`](https://github.com/obra/superpowers) skill when they touch non-overlapping files.

**When a new session picks this up:**

1. Read this plan
2. Read [`docs/specs/2026-04-07-mailing-list-cli-design.md`](../specs/2026-04-07-mailing-list-cli-design.md) §3 (data model), §5 (send pipeline), §9 (compliance), §13 (email-cli interface)
3. Pick the next unstarted phase (Phase 3 is next)
4. Write the implementation plan for that phase using the `superpowers:writing-plans` skill
5. Execute the plan
6. Tag the release

---

## 8. Open questions to revisit during Phase 5

These are not blockers; they're decisions to lock in when broadcasts actually ship:

1. **Native broadcasts vs batch send.** `email-cli broadcast create --send` gives us auto-wired per-recipient `{{{RESEND_UNSUBSCRIBE_URL}}}` but requires one Resend segment per campaign (which is cheap). `email-cli batch send --file` is fallback for edge cases. Default to native; fall back on error.
2. **Broadcast-owned segments.** When `broadcast create` targets a mailing-list-cli segment (a saved filter), do we materialize the filter into a fresh Resend segment at send time, or do we just pass individual recipient IDs to batch send? The native path forces the former. Probably the right answer, but the trade-off is one Resend segment per broadcast.
3. **Click tracking domain.** Resend's default click tracker is `resend.dev`. Some users want `tracking.yourdomain.com` — defer until anyone asks.
4. **Template shell + markdown body.** Research dossier 05 recommends a hybrid MJML shell + Markdown body path where the agent writes markdown and the CLI wraps it in a chosen MJML shell. Phase 4 ships pure MJML; the hybrid lands as v0.2 if demanded.

---

## 9. What you get at each milestone

| Version | After this phase you can… |
|---|---|
| **v0.0.3** (shipped) | Create lists, add contacts, shell out to `email-cli` for Resend sync. That's it. |
| **v0.0.4** (Phase 3) | Tag contacts, set custom fields, import a 10k CSV, save boolean-filter segments, list contacts matching a filter. |
| **v0.1.0** (Phase 4) | Author MJML templates with an LLM using `template guidelines`, lint them, preview them rendered. Your "templates with explicit guidelines" ask is DONE. |
| **v0.1.1** (Phase 5) | Create a named broadcast, target a list or segment, schedule or send immediately. Your "segment batches and give them names" ask is DONE. |
| **v0.1.2** (Phase 6) | See bounce rate, click count per link, unsubscribe count per campaign, opens, CTR, deliverability dashboard. Your "bounce rate tracking", "unsubscribe count per batch", and "click tracking" asks are DONE. |
| **v0.1.3** (Phase 7) | Run double opt-in flows, honor unsubscribes across all campaigns, GDPR-erase contacts, verify SPF/DKIM/DMARC. |
| **v0.1.4** (Phase 8) | Run A/B tests with auto-winner promotion. Run the daemon for scheduled sends. |
| **v0.1.5** (Phase 9) | Install via Homebrew or Cargo from prebuilt binaries. |

**At v0.1.2, all five of your original asks are shipped.** That's the target.

---

## 10. If anything else needs to be added to email-cli

The team shipped all three MUST asks in v0.6.2 (see [`docs/email-cli-gap-analysis.md`](../email-cli-gap-analysis.md)). If during Phase 6 we discover we need polling support with a richer filter — e.g. `email list --since <ts> --type <event>` — we file that as a new ask. Otherwise nothing else should be needed from the email-cli team through v0.1.5.

**The architectural rule still holds: `mailing-list-cli` has zero Resend HTTP code. Every Resend touchpoint flows through `email-cli`.**

---

*End of parity plan.*
