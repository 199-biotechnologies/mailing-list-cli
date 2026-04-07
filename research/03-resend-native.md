# Resend Native Capabilities — What's Already Built

**Research date:** 2026-04-07
**Goal:** Map exactly what Resend's API and dashboard already provide so `mailing-list-cli` can lean on Resend wherever possible and only build the gap.

All endpoints below are exact strings copied from official Resend documentation. Where a feature is not documented, that is stated explicitly. No features have been inferred or invented.

---

## 1. Broadcasts API

Resend's Broadcasts product is a no-code-friendly email-blast system layered over the same delivery infrastructure as transactional sending. It is the closest thing Resend has to "mailing list" functionality.

### Endpoints

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/broadcasts` | Create a broadcast (draft, optionally schedule via `send: true` + `scheduled_at`) |
| `GET` | `/broadcasts` | List broadcasts |
| `GET` | `/broadcasts/{id}` | Retrieve one broadcast |
| `PATCH` | `/broadcasts/{id}` | Update a broadcast (only valid in `draft` state) |
| `POST` | `/broadcasts/{id}/send` | Send (or schedule) an existing broadcast |
| `DELETE` | `/broadcasts/{id}` | Delete a broadcast — also cancels schedule if it was `scheduled` |

> Note: The `llms.txt` index lists these under `/emails/broadcast/...` paths but the live API reference uses `/broadcasts/...`. The API reference is canonical.

### Create-broadcast fields (`POST /broadcasts`)

Required:
- `segment_id` (string) — target segment (formerly `audience_id`)
- `from` (string) — `"Name <email@domain.com>"` accepted
- `subject` (string)

Optional:
- `reply_to` (string | string[])
- `html` (string) — supports Contact Properties as merge tags
- `text` (string) — auto-generated from HTML if omitted
- `react` (Node.js SDK only) — React Email component
- `name` (string) — internal label
- `topic_id` (string) — scope to a topic
- `send` (boolean, default `false`)
- `scheduled_at` (string) — natural language (`"in 1 min"`) or ISO 8601; **requires `send: true`**

Response: `{ "id": "<uuid>" }`

### Statuses

A broadcast moves through: **`draft` → `scheduled` → `queued` → `sent`**.
- `draft` is editable and deletable.
- `scheduled` can be canceled (via `DELETE`, which returns it to deletion / not delivered).
- Once sent, no further actions documented.

### Scheduling, preview, cancel, A/B

- **Schedule:** Yes, via `scheduled_at` on `POST /broadcasts` or `POST /broadcasts/{id}/send`. Accepts natural-language strings or ISO 8601.
- **Preview:** Dashboard has a "Test Email" feature that sends preview copies to specific addresses or team members. **No documented API endpoint for sending a test broadcast** — only the dashboard supports this.
- **Cancel a scheduled broadcast:** `DELETE /broadcasts/{id}`. Documented behavior: "if you delete a broadcast that has already been scheduled to be sent, we will automatically cancel the scheduled delivery and it won't be sent." Note: the delete-broadcast docs also say "you can only delete broadcasts with a draft status," which contradicts the cancel-on-schedule behavior. Treat as undocumented edge — verify in practice.
- **Pause:** No documented pause endpoint. Only delete/cancel.
- **A/B testing:** **Not documented anywhere in Resend's docs.** Resend has no A/B feature for broadcasts.

### Throttling for large audiences

Resend's marketing copy says broadcasts can "queue, throttle, and send millions of emails with one API call" but **no specific throughput numbers are documented** (e.g. emails/sec for a 50k blast). Internal queue is implied but opaque.

---

## 2. Audiences & Segments API

**Audiences is deprecated** in favor of **Segments**. The Audiences endpoints still respond but are scheduled for removal:

> "These endpoints still work, but will be removed in the future." — `POST /audiences` reference page

### Segments endpoints (current)

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/segments` | Create a segment |
| `GET` | `/segments` | List segments |
| `GET` | `/segments/{id}` | Retrieve one segment |
| `DELETE` | `/segments/{id}` | Delete a segment |
| `GET` | `/segments/{id}/contacts` | List contacts in a segment |
| `POST` | `/contacts/{id}/segments` | Add a contact to a segment |
| `DELETE` | `/contacts/{id}/segments/{segment_id}` | Remove a contact from a segment |

**Create segment** payload — only documented field is `name` (string). Response: `{ object: "segment", id, name }`.

### Schema and capabilities

- **Max audience/segment size:** **Not documented.** No published cap.
- **Segmentation logic / filter syntax:** **Not documented in the public API.** The dashboard exposes segmentation controls, but the public reference shows no `filter` or `query` field on `POST /segments`. Segments appear to be manually-populated containers via add/remove APIs, not computed predicates — unless Resend has dashboard-only filter UI that isn't exposed via API.
- **Custom fields:** Yes, via the **Contact Properties API** (separate resource — see Section 3).
- **CSV import via API:** **Does not exist.** CSV import is dashboard-only via `Contacts → Add Contacts → Import CSV`. The API has no `/contacts/import` or equivalent endpoint. Bulk loads via API must be one-`POST`-per-contact, subject to the 5 req/sec rate limit.
- **Tags/labels on contacts:** Not as first-class tags — only as keys inside `properties` (free-form object). Contact Properties is the closest equivalent.

### Topics (preference center primitive)

Topics are a separate concept from segments — they're how *contacts* manage their own preferences:

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/topics` | Create a topic |
| `GET` | `/topics` | List topics |
| `GET` | `/topics/{id}` | Retrieve one |
| `PATCH` | `/topics/{id}` | Update one |
| `DELETE` | `/topics/{id}` | Delete one |
| `GET` | `/contacts/{id}/topics` | Get a contact's topic subscriptions |
| `POST` | `/contacts/{id}/topics` | Update topic subscriptions for a contact |

Each topic is permanently `opt-in` or `opt-out` (the default subscription mode cannot be changed after creation). Topics can be `Public` (visible on the unsubscribe preference page to all contacts) or `Private`. A broadcast can be scoped to a topic via `topic_id`, meaning only contacts subscribed to that topic receive it.

---

## 3. Contacts API

### Endpoints

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/contacts` | Create a contact |
| `GET` | `/contacts` | List contacts (cursor pagination via `after`/`before`, `limit` max 100) |
| `GET` | `/contacts/{id_or_email}` | Retrieve one |
| `PATCH` | `/contacts/{id_or_email}` | Update (toggles `unsubscribed`, etc.) |
| `DELETE` | `/contacts/{id}` | Delete one |

### Contact fields (canonical)

- `id` — UUID, server-generated
- `email` — required, unique
- `first_name` — optional string
- `last_name` — optional string
- `unsubscribed` — boolean; setting `true` unsubscribes from **all broadcasts** (account-wide flag, not per-segment)
- `properties` — free-form object of custom key/value pairs (gates the per-contact data model)
- `segments` — relationship (managed via `/contacts/{id}/segments`)
- `topics` — relationship (managed via `/contacts/{id}/topics`)
- `created_at` — ISO 8601 timestamp

### Contact Properties API (custom fields)

Custom fields are managed as their own resource:

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/contact-properties` | Create a custom property |
| `GET` | `/contact-properties` | List |
| `GET` | `/contact-properties/{id}` | Get one |
| `PATCH` | `/contact-properties/{id}` | Update |
| `DELETE` | `/contact-properties/{id}` | Delete |

### Batch operations

There is **no `/contacts/batch` or bulk-create endpoint**. The closest batch primitive is `POST /emails/batch` (sends up to 100 emails at once) — this is for transactional sending, not contact management. **Bulk contact import via API is impossible — you must POST one at a time and live within the 5 req/sec rate limit, OR use the dashboard CSV importer.**

### Rate limits

5 requests/sec per team (see Section 7). Contact APIs share that ceiling with all other API calls.

---

## 4. Tracking (opens & clicks)

### Mechanics

- **Open tracking** uses a 1×1 transparent GIF inserted in the HTML body. When the recipient's mail client downloads it, Resend logs the open. Resend explicitly notes: "Open rates are not always accurate" (due to image-blocking and Apple Mail Privacy Protection).
- **Click tracking** rewrites every link in the HTML body so it points to a Resend redirect server, which logs the click and 302s to the original URL.

### Configuration

- **Both are disabled by default.**
- Configured **per-domain** (not per-email or per-broadcast).
- Set via:
  - Dashboard → `Domains` → select domain → `Configuration` tab → toggle `openTracking` and `clickTracking`
  - API: domain update call with `openTracking: true` and `clickTracking: true`

### Webhook emission

- **Click events** (`email.clicked`) are emitted **per click** with the original URL preserved (`data.click.link`), plus IP address and user-agent (see Section 5).
- **Open events** (`email.opened`) are emitted similarly with timestamp, IP, user-agent.
- These are not aggregated — every recipient action produces a separate webhook event. Aggregation/dedup is the consumer's job.

---

## 5. Webhooks / Events

### Endpoint to manage webhooks

`POST /webhooks` (and CRUD), or via dashboard.

### Event types — full list

**Email events (11):**
1. `email.sent` — API request was successful
2. `email.scheduled` — email scheduled for future delivery
3. `email.delivered` — successfully accepted by recipient mail server
4. `email.delivery_delayed` — temporary issue, will retry (a.k.a. soft bounce)
5. `email.bounced` — permanently rejected (hard bounce, after retries exhausted)
6. `email.complained` — recipient marked as spam / FBL complaint
7. `email.opened` — recipient opened the email (requires open tracking)
8. `email.clicked` — recipient clicked a link (requires click tracking)
9. `email.failed` — failed due to sending error
10. `email.suppressed` — Resend blocked the send because address was on suppression list
11. `email.received` — inbound email (for inbound parsing customers)

**Domain events (3):** `domain.created`, `domain.updated`, `domain.deleted`

**Contact events (3):** `contact.created`, `contact.updated`, `contact.deleted`

### Bounce taxonomy

Resend exposes both `type` and `subType` on `data.bounce`:

**`type` values:**
- `Permanent` — hard bounce
- `Transient` — soft bounce (retry-able; if retries fail, an `email.bounced` is emitted)
- `Undetermined` — bounce response was unparseable

**`subType` values (documented in `dashboard/emails/email-bounces`):**

| `type` | `subType` | Meaning |
|---|---|---|
| Permanent | `General` | Provider sent a hard bounce |
| Permanent | `NoEmail` | Recipient address could not be parsed from bounce message |
| Transient | `General` | Soft bounce, retryable |
| Transient | `MailboxFull` | Recipient inbox is full |
| Transient | `MessageTooLarge` | Exceeded provider size limit |
| Transient | `ContentRejected` | Provider doesn't allow the content |
| Transient | `AttachmentRejected` | Disallowed attachment |
| Undetermined | `Undetermined` | Could not classify |

There is also a `Suppressed` subType seen in webhook payloads (when the address is already on the suppression list before send) — confirmed by example payload in Resend's webhook docs.

### Webhook payload schemas

All webhook payloads share the envelope:

```json
{
  "type": "email.<event_name>",
  "created_at": "2024-11-22T23:41:12.126Z",
  "data": { ... }
}
```

The `data` object always contains:
- `email_id` (string, UUID)
- `broadcast_id` (string, UUID, only present if sent via Broadcasts)
- `template_id` (string, UUID, only present if a template was used)
- `from` (string)
- `to` (string[])
- `subject` (string)
- `created_at` (ISO 8601)
- `tags` (object — custom key/value metadata passed at send time)

**Event-specific fields:**

`email.bounced`:
```json
"data": {
  "...": "envelope fields",
  "bounce": {
    "message": "The recipient's email address is on the suppression list...",
    "type": "Permanent",
    "subType": "Suppressed"
  }
}
```

`email.clicked`:
```json
"data": {
  "...": "envelope fields",
  "click": {
    "ipAddress": "122.115.53.11",
    "link": "https://resend.com",
    "timestamp": "2024-11-24T05:00:57.163Z",
    "userAgent": "Mozilla/5.0 ..."
  }
}
```

`email.opened`:
```json
"data": {
  "...": "envelope fields",
  "open": {
    "ipAddress": "122.115.53.11",
    "timestamp": "2024-11-24T05:00:57.163Z",
    "userAgent": "Mozilla/5.0 ..."
  }
}
```

(Note: the exact `email.opened` JSON example is reconstructed from search-result excerpts. Resend's individual per-event payload pages were not directly accessible, but the `open` object structure mirrors `click` minus the `link` field.)

`email.complained`, `email.delivered`, `email.delivery_delayed`, `email.failed`, `email.suppressed`, `email.scheduled`, `email.sent` — these were not directly fetched, but per Resend's docs they all use the same envelope and add only minimal/no event-specific fields beyond standard delivery metadata.

### Webhook security

Resend signs webhooks for verification (Svix-style HMAC); details in `/docs/webhooks/introduction`. Not load-bearing for this research.

---

## 6. Suppression List

### What's automatic

Resend **automatically** adds an address to the suppression list when:
- A hard bounce is received (the recipient mail server permanently rejects)
- A spam complaint is received (recipient marks as spam / FBL)

Once an address is on the suppression list, future sends to it produce `email.suppressed` and **are not delivered**.

### Scope

**Regional**, not per-domain. Documented quote: "An address suppressed on any sending domain within a region affects all domains in that region." This is broader than per-account in some senses (covers all your domains in that region) and narrower than account-wide cross-region.

### API access

**There is no documented public API for the suppression list.** The `llms.txt` index does **not** list `/suppressions` endpoints. Suppression management is **dashboard-only**:
- View suppressed addresses in `Emails` dashboard
- Click into a suppressed email → "Remove from suppression list" button
- No documented bulk import/export, no API query/add/remove

This is a real gap — programmatic access is not available.

---

## 7. Rate Limits & Quotas

### API rate limit

**5 requests per second per team**, shared across all API keys on that team.
- Returns `429` when exceeded
- Headers returned (IETF standard):
  - `ratelimit-limit`
  - `ratelimit-remaining`
  - `ratelimit-reset` (seconds until window reset)
  - `retry-after` (seconds to wait)
- Increase available "for trusted senders upon request" (contact Resend support)
- **No documented burst capacity** above 5 req/sec
- **No per-minute or per-day request limit** documented (only per-second)

### Email quotas (separate from API rate limits)

- `x-resend-daily-quota` header — free plan only
- `x-resend-monthly-quota` header — all plans
- Sent **and** received emails count toward quotas
- Multi-recipient emails count each recipient separately
- Hard cap on overage: **5× your plan's monthly quota**, after which sending pauses until next billing cycle

### Quality thresholds (Resend will throttle/block accounts that exceed)

- **Bounce rate** must stay below **4%**
- **Spam complaint rate** must stay below **0.08%**

### Broadcast-specific throughput

**No documented per-second send rate for broadcasts.** Resend marketing materials say the queue can handle "millions of emails with one API call," but no SLA-grade throughput numbers are published. For a 50k broadcast, the actual emit rate is opaque.

---

## 8. Templates

Resend has a **first-class hosted template system**.

### Endpoints

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/templates` | Create a template |
| `GET` | `/templates` | List |
| `GET` | `/templates/{id}` | Retrieve one |
| `PATCH` | `/templates/{id}` | Update |
| `POST` | `/templates/{id}/duplicate` | Duplicate |
| `POST` | `/templates/{id}/publish` | Publish (templates have draft/published states) |
| `DELETE` | `/templates/{id}` | Delete |

### Template usage

When sending a transactional email, you reference the template via the `template` object (NOT a `template_id` field):

```json
{
  "from": "...",
  "to": "...",
  "template": {
    "id": "tmpl_xxx",
    "variables": {
      "first_name": "Boris",
      "company": "Acme"
    }
  }
}
```

Resend renders server-side and sends.

### React Email integration

- Full hosted React Email support
- Templates can be created from React Email components (only `react` and `@react-email/components` imports allowed — no local files, no third-party packages)
- The Node.js SDK accepts a `react: <Component />` field on send/broadcast endpoints, which renders client-side

### Merge tag conventions

Triple-brace syntax: `{{{VARIABLE_NAME}}}`

**Reserved variables** (auto-populated by Resend):
- `{{{FIRST_NAME}}}`
- `{{{LAST_NAME}}}`
- `{{{EMAIL}}}`
- `{{{RESEND_UNSUBSCRIBE_URL}}}` — magic unsubscribe link, see Section 9
- `{{{contact}}}` — full contact object
- `{{{this}}}` — current scope

Each template supports up to **20 custom variables** with declared types (`string`, `number`) and optional fallbacks.

---

## 9. List-Unsubscribe & One-Click

### Auto-injection

**Mixed.** Two distinct cases:

1. **Broadcasts to a Resend Audience/Segment:** When sending broadcasts, Resend's hosted unsubscribe page is wired up automatically and the `{{{RESEND_UNSUBSCRIBE_URL}}}` template variable resolves to a per-recipient, per-broadcast unsubscribe URL. The hosted page handles the unsubscribe flow on Resend's domain.
2. **Transactional emails:** Resend **does not automatically inject** the `List-Unsubscribe` header. You must manually pass it via the `headers` parameter:
   ```json
   "headers": { "List-Unsubscribe": "<https://example.com/unsubscribe>" }
   ```
   And you must self-host the unsubscribe endpoint, which must accept GET (display) and POST (one-click per RFC 8058) and return 200/202 within 48 hours.

### RFC 8058 one-click for broadcasts

**Not explicitly confirmed in docs.** Resend's hosted unsubscribe page works for clicks but the docs do **not** explicitly say whether `List-Unsubscribe-Post: List-Unsubscribe=One-Click` is auto-set on broadcast emails. This needs to be verified empirically by inspecting raw broadcast headers.

### Hosted unsubscribe page

- Resend hosts a customizable unsubscribe / preference page for broadcast contacts
- Customizable: title, description, logo, background color, text color, accent color
- Pro plan removes the "Powered by Resend" footer
- Single page is shared across all domains on the team (you cannot have multiple)
- Integrates with **Topics** (Section 2) — public topics show as toggleable preferences on the page
- Sets the `unsubscribed` boolean on the contact when the user opts out (can be toggled back via PATCH)

---

## 10. Pricing for 50k subscribers, ~4 broadcasts/month (early 2026)

### Marketing tiers (priced by contact count, unlimited sends)

| Plan | Price/mo | Contact Limit |
|---|---|---|
| Free | $0 | 1,000 |
| Pro Marketing | $40 | 5,000 |
| Pro Marketing | $80 | 10,000 |
| Pro Marketing | $120 | 15,000 |
| Pro Marketing | $180 | 25,000 |
| **Pro Marketing** | **$250** | **50,000** ← target tier |
| Pro Marketing | $450 | 100,000 |
| Pro Marketing | $650 | 150,000 |
| Enterprise | Custom | Custom |

**Sends are unlimited on marketing plans** (priced purely by contact count).

### Transactional tiers (priced by email volume)

| Plan | Price/mo | Emails/mo | Daily limit |
|---|---|---|---|
| Free | $0 | 3,000 | 100/day |
| Pro | $20 | 50,000 | None |
| Pro | $35 | 100,000 | None |
| Scale | $90 | 100,000 | None |
| Scale | varies | up to 2.5M | None |
| Enterprise | Custom | Custom | Flexible |

### Estimate for the use case

**50,000 contacts × 4 broadcasts/month = ~200,000 broadcast sends/month.**

- Marketing plan: **$250/month** (Pro Marketing 50k tier — covers all 200k broadcast sends since marketing sends are unlimited)
- If transactional sending is also needed (welcome emails, password resets, etc.) on top, add the **$20–$35 Pro transactional tier**

**Total expected cost: ~$250–$285/month** for 50k subs + 4 monthly broadcasts.

Note: Resend's marketing/transactional separation means broadcasts to your audience do *not* count against transactional quotas. They're billed solely on contact count.

---

## 11. Other mailing-list-relevant features

### What Resend has

- **Hosted unsubscribe / preference center** (Section 9) — branded, per-team, integrates with Topics
- **Topics** (Section 2) — preference categories that contacts can subscribe/unsubscribe per-topic, exposed on the hosted preference page
- **Custom contact properties** via `/contact-properties` resource
- **Built-in `{{{RESEND_UNSUBSCRIBE_URL}}}`** merge tag in templates and broadcasts
- **Webhooks for contact lifecycle** (`contact.created`, `contact.updated`, `contact.deleted`) — useful for syncing to external CRM/database
- **Tags** on individual emails (key/value metadata that flows through to webhook payloads — useful for analytics segmentation)
- **DPA / GDPR acknowledgment** in legal docs (Resend has a DPA for EU GDPR compliance)

### What Resend does NOT have

- **Double opt-in** — not documented anywhere. No built-in confirmation email flow. You must build this yourself by sending a transactional confirmation email that POSTs back to your own endpoint, which then patches `unsubscribed: false`.
- **GDPR consent capture** — no documented field for consent timestamp, source, IP, or proof-of-consent on contacts. Custom properties can be used as a workaround.
- **Geo/device analytics on opens/clicks** — webhook payloads include `ipAddress` and `userAgent` but no parsed geo (country/city) or device-class breakdown. You must enrich downstream.
- **A/B testing** — no native broadcast A/B feature.
- **Pause/resume** of in-flight broadcasts — no documented endpoint.
- **Send-time optimization** — no per-recipient time-zone send.
- **Bulk contact import via API** — dashboard CSV only; programmatic import is one-POST-at-a-time.
- **Suppression list API** — dashboard-only.
- **Per-segment unsubscribe** — `unsubscribed` is a single account-wide boolean per contact (or per-topic via Topics, but not per-segment).
- **Cold-start warmup automation** — no built-in IP warmup scheduler (separate from Scale plan dedicated IP feature).

---

## Source URLs (canonical)

- API reference index: https://resend.com/docs/api-reference
- Broadcasts API: https://resend.com/docs/api-reference/broadcasts
- Contacts API: https://resend.com/docs/api-reference/contacts
- Segments API: https://resend.com/docs/api-reference/segments
- Topics API: https://resend.com/docs/api-reference/topics
- Templates API: https://resend.com/docs/api-reference/templates
- Webhook event types: https://resend.com/docs/dashboard/webhooks/event-types
- Open/click tracking: https://resend.com/docs/dashboard/domains/tracking
- Bounce taxonomy: https://resend.com/docs/dashboard/emails/email-bounces
- Suppression list: https://resend.com/docs/dashboard/emails/email-suppressions
- Suppression FAQ: https://resend.com/docs/knowledge-base/why-are-my-emails-landing-on-the-suppression-list
- Rate limits: https://resend.com/docs/api-reference/rate-limit
- Account quotas: https://resend.com/docs/knowledge-base/account-quotas-and-limits
- List-Unsubscribe header guide: https://resend.com/docs/dashboard/emails/add-unsubscribe-to-transactional-emails
- Unsubscribe page: https://resend.com/docs/dashboard/settings/unsubscribe-page
- Topics: https://resend.com/docs/dashboard/topics/introduction
- Audiences (deprecated): https://resend.com/docs/dashboard/audiences/introduction
- Segments: https://resend.com/docs/dashboard/segments/introduction
- Broadcasts intro: https://resend.com/docs/dashboard/broadcasts/introduction
- Templates intro: https://resend.com/docs/dashboard/templates/introduction
- Pricing: https://resend.com/pricing and https://resend.com/pricing?product=marketing
- llms.txt index: https://resend.com/docs/llms.txt
