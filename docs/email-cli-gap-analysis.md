# mailing-list-cli ↔ email-cli Gap Analysis (Claude)

**Date:** 2026-04-07
**Method:** Cross-referenced mailing-list-cli's design spec (§13 "The email-cli Interface"), the Resend native capabilities research dossier (`research/03-resend-native.md`), and email-cli's `src/cli.rs` clap definitions fetched from GitHub.

## Summary

email-cli covers **about 70% of what mailing-list-cli's v0.1 needs**. The remaining 30% breaks into three buckets: hard blockers (must add to email-cli), workarounds (functional but suboptimal), and Resend-side gaps (nobody can fix).

Three additions to email-cli would fully unblock mailing-list-cli's v0.1:

1. **`contact create --properties <json>`** — to sync custom fields to Resend
2. **`broadcast create / send / cancel / get`** — to use Resend's native Broadcasts API
3. **`events list --since <ts> [--type <t>]`** — to let mailing-list-cli poll instead of running its own webhook listener

Everything else is either optional or already covered.

## Coverage table

### ✅ EXISTS in email-cli (mailing-list-cli already uses these)

| Capability | email-cli command | Resend endpoint |
|---|---|---|
| Capability discovery | `email-cli --json agent-info` | (local) |
| Profile health check | `email-cli --json profile test <name>` | (validates API key) |
| Audience CRUD (deprecated) | `email-cli --json audience {list,get,create,delete}` | `/audiences` |
| Contact CRUD (basic) | `email-cli --json contact {list,get,create,update,delete} --audience <id>` | `/contacts` |
| Single send | `email-cli --json send --to --subject --html --text` | `POST /emails` |
| Batch send | `email-cli --json batch send --file <json>` | `POST /emails/batch` |
| Domain CRUD | `email-cli --json domain {list,get,create,verify,delete,update}` | `/domains` |
| Domain tracking toggle | `email-cli --json domain update <id> --open-tracking --click-tracking` | `PATCH /domains/{id}` |
| API key CRUD | `email-cli --json api-key {list,create,delete}` | `/api-keys` |
| Webhook listener (receiver) | `email-cli webhook listen --port <p>` | (local HTTP server) |
| Outbox / durable retry | `email-cli outbox {list,retry,flush}` | (local queue) |

### ⚠️ GAPS — email-cli does NOT wrap, but Resend does

| # | Capability | Resend endpoint(s) | Missing email-cli command | Priority | Why mailing-list-cli needs it |
|---|---|---|---|---|---|
| 1 | **Custom contact properties** | `/contacts` body has free-form `properties` object; dedicated `/contact-properties` resource | `contact create --properties '{"company":"acme"}'` plus `contact-property {list,create,delete}` | **MUST** | Without this, mailing-list-cli's custom field schema is local-only. Contacts on Resend have no extra fields, so the hosted preference center can't show them and Liquid merge tags in Resend templates can't reference them. mailing-list-cli currently stores custom fields in its own SQLite as a workaround, but this loses the round-trip with Resend. |
| 2 | **Resend Broadcasts API** | `POST /broadcasts`, `POST /broadcasts/{id}/send`, `DELETE /broadcasts/{id}`, `GET /broadcasts`, `PATCH /broadcasts/{id}` | `email-cli broadcast {create,list,get,send,delete}` with `--audience-id`, `--from`, `--subject`, `--html`, `--scheduled-at`, `--topic-id` | **MUST** | mailing-list-cli currently uses `email-cli batch send` to send campaigns, which works but bypasses Resend's per-recipient `{{{RESEND_UNSUBSCRIBE_URL}}}` magic and the hosted unsubscribe page wiring. Native Broadcasts also gives us server-side scheduling/cancel and the rate-limit-aware queue Resend manages internally. |
| 3 | **Bulk events query** | (no global Resend API; populated by webhooks into email-cli's local DB) | `email-cli events list --since <rfc3339> [--type <t>] [--account <a>] [--limit N]` against the local events table | **MUST** | email-cli's current `events list` only takes `--message <id>`. mailing-list-cli has to run its own webhook listener as a workaround, which means two listeners share a Resend webhook and the user has to set up two webhook URLs. If email-cli grew this command we could remove the entire webhook listener subsystem from mailing-list-cli. |
| 4 | **Resend Segments API (modern)** | `POST/GET/DELETE /segments`, `POST /contacts/{id}/segments`, `DELETE /contacts/{id}/segments/{sid}` | `email-cli segment {list,create,delete,members}` and `email-cli contact segment {add,remove}` | SHOULD | Resend deprecated Audiences in favor of Segments. email-cli still uses the deprecated `/audiences` paths. mailing-list-cli treats lists as "audiences" today, but if Resend removes the deprecated endpoints, both binaries break. Migrating email-cli to Segments is overdue. |
| 5 | **Resend Topics API (preference center)** | `POST/GET/PATCH/DELETE /topics`, `POST /contacts/{id}/topics` | `email-cli topic {list,create,delete}` and `email-cli contact topic {set,unset}` | SHOULD | Topics are the only way to expose granular subscription preferences on Resend's hosted preference page. mailing-list-cli currently can't offer "subscribe to weekly digest only" UX without these. |
| 6 | **Webhook configuration CRUD** | `POST/GET/PATCH/DELETE /webhooks` | `email-cli webhook {register,list,delete} --url --events` | SHOULD | email-cli has `webhook listen` (the receiver) but you have to configure the Resend-side webhook through the dashboard. mailing-list-cli's `dnscheck`/`health` flow can't fully validate the wiring without this. |
| 7 | **Resend Templates API** | `POST/GET/PATCH/DELETE /templates`, publish, duplicate | `email-cli template {list,create,publish,delete}` | NICE | mailing-list-cli compiles MJML locally via `mrml`, so we don't need this. But if a user prefers Resend's hosted React Email templates, this would let them use them through the same wrapper. |
| 8 | **Custom tracking subdomain** | `PATCH /domains/{id}` with custom tracking-subdomain config | `email-cli domain update <id> --tracking-domain tracking.example.com` | NICE | email-cli's `domain update` only flips `--open-tracking` and `--click-tracking` booleans. Some senders want their click-tracking links on `tracking.example.com` instead of resend.dev. |
| 9 | **Per-recipient tags on send** | `tags` array on `POST /emails` and `POST /emails/batch` (key/value pairs that flow through to webhook payloads) | `email-cli send … --tag key=value` and `--tag` arrays in batch JSON | NICE | mailing-list-cli will want to tag every send with `broadcast_id=42` so the webhook events can be linked back to the campaign. The batch JSON file format could already accept `tags`, but `email-cli send` should expose `--tag` flags too. |

### ❌ NOT IN RESEND — nobody can fix these

| Capability | What it would do | Why it can't be added |
|---|---|---|
| Suppression list programmatic access | Read/write/import the global account suppression list | Resend's `/suppressions` is dashboard-only; not in any docs or `llms.txt` index. mailing-list-cli maintains its own local mirror from webhook events. |
| Bulk contact import via API | `POST /contacts/import` accepting CSV | Resend has no batch contact endpoint at all. CSV import is dashboard-only. mailing-list-cli's `contact import` will chunk into individual `contact create` calls under the 5 req/sec rate limit. |
| Pause/resume in-flight broadcast | Pause an actively-sending broadcast | No documented endpoint. Only `DELETE /broadcasts/{id}` for scheduled (not in-flight) broadcasts. |
| Per-segment unsubscribe | Unsubscribe contact from one segment but not all | `unsubscribed` is a single account-wide boolean per contact. Topics provide per-topic unsubscribe but not per-segment. |
| Geo / device enrichment on opens | `country: 'DE'`, `device: 'iPhone'` on open events | Webhook payloads carry only `ipAddress` and `userAgent`. Enrichment is downstream work in mailing-list-cli (likely via `maxminddb`). |
| Predictive analytics, RFM, send-time optimization | Best-time-to-send, churn risk, etc. | Resend doesn't compute these. They'd be a mailing-list-cli feature, not an email-cli feature. |

## Recommended email-cli upgrades, ranked

If the email-cli team wants to do exactly **one** thing for mailing-list-cli, it's #1. If they want to do exactly **three** things, it's #1 + #2 + #3 from the GAPS table above.

| Priority | Upgrade | Effort estimate | mailing-list-cli benefit |
|---|---|---|---|
| 1 | `contact create --properties <json>` (and round-tripping `properties` in `contact get`/`update`) | Small (just plumbing fields through to the existing `/contacts` request body) | Unblocks the entire custom-field surface in v0.1; mailing-list-cli stops being a black-box for Resend's preference center |
| 2 | Wrap Resend Broadcasts API: `email-cli broadcast {create,list,get,send,cancel}` | Medium (new noun, ~6 commands, mostly thin wrappers) | mailing-list-cli's send pipeline gets per-recipient `{{{RESEND_UNSUBSCRIBE_URL}}}` automatically + server-side scheduling/cancel |
| 3 | `email-cli events list --since <rfc3339> --type <t>` against the local DB | Small (read against existing events table that `webhook listen` populates) | mailing-list-cli can drop its own webhook listener entirely; one Resend webhook URL instead of two |
| 4 | Migrate Audiences → Segments + add Topics | Medium (new noun set, schema changes) | Future-proofs both binaries against Resend's audiences deprecation; unlocks granular subscription preferences |
| 5 | Webhook CRUD (`webhook register/list/delete`) | Small | One-shot wiring instead of dashboard clicks |
| 6 | `--tag key=value` on send / batch send | Tiny | mailing-list-cli can attribute events back to broadcasts cleanly |

## What this means for mailing-list-cli right now (without email-cli changes)

mailing-list-cli's v0.1 spec is shippable on the current email-cli surface, with three known compromises:

1. **Custom fields are local-only.** A custom field defined in `mailing-list-cli field create company --type text` exists only in mailing-list-cli's SQLite. The Resend-side contact has no `properties.company`. Merge tags in MJML templates work because we render locally before passing HTML to email-cli, so this is invisible to the end user — but the Resend dashboard will show contacts with no custom data.

2. **Broadcasts use batch send instead of `/broadcasts`.** This means mailing-list-cli has to inject its own per-recipient `List-Unsubscribe` header (via the `headers` field in the batch JSON), pointing at its own webhook listener. The hosted Resend unsubscribe page is bypassed. Functional, but loses some Resend value.

3. **Two webhook listeners run.** email-cli's `webhook listen` handles inbox events (`email.received`). mailing-list-cli runs a separate listener for delivery events (`email.bounced`, `email.complained`, `email.opened`, `email.clicked`, etc.). The user has to configure two webhook URLs in Resend. Annoying but workable.

If email-cli adds gaps #1, #2, and #3, mailing-list-cli simplifies dramatically and gets first-class Resend integration.
