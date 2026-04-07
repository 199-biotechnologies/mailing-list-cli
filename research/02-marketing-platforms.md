# Marketing Email Platforms: Mailchimp, MailerLite, Kit (ConvertKit)

Research date: 2026-04-07. Scope: feature parity, data models, segmentation, automations,
A/B testing, personalization, compliance, templates, APIs, and weaknesses — to inform the
data model and command surface of `mailing-list-cli` (Resend-backed, agent-friendly).

---

## 1. The 80/20 minimum viable feature set

After cross-referencing the three platforms, an SMB marketer cannot live without:

1. **Contacts** with email + status (active / unsubscribed / bounced / cleaned).
2. **Custom fields** (typed: text, number, date) attached to each contact.
3. **Tags** — manual / automation-applied labels (n:m with contacts).
4. **Lists or "audiences"** (Mailchimp's hard boundary) OR a single list with **groups**
   (MailerLite) OR a single list with tags-only (Kit). The unified pattern: a stable
   bucket the user explicitly puts subscribers into.
5. **Dynamic segments** that auto-evaluate from boolean rules over fields, tags,
   engagement, and dates. Match modes: ALL / ANY / NONE.
6. **Forms** (signup) with double opt-in switch.
7. **Broadcast / campaign** sender with subject, from, body, recipient filter, schedule.
8. **Automations / sequences**: a DAG of `trigger -> wait -> condition -> action`
   nodes. Triggers: form submit, tag added, date, custom field change. Actions:
   send email, add/remove tag, update field, copy/move to group, webhook.
9. **Templates** (block-based and raw HTML escape hatch) + **merge tags** for
   personalization.
10. **A/B test** of at least subject line and content, with auto-pick by open or click rate.
11. **Bounce/complaint auto-suppression** (hard bounce → cleaned; complaint → suppressed).
12. **List-Unsubscribe header (RFC 8058 one-click)** auto-injected — non-negotiable
    since the Feb/Jun 2024 Gmail/Yahoo enforcement.
13. **Reports**: opens, clicks, bounces, unsub, per-broadcast and per-link.
14. **REST API** with token auth, JSON, cursor pagination, batch endpoint, webhooks.

Anything beyond this — predictive analytics, AI subject-line writers, e-commerce
recommendations, multivariate testing, SMS — is **upsell territory**, not MVP.

---

## 2. Data model comparison

### 2.1 Mailchimp

| Concept | Notes |
|---|---|
| **Audience** (List) | Hard top-level container. Contacts in different audiences are billed separately and treated as different people. Most accounts should run a single audience — multi-audience is widely considered an anti-pattern. |
| **Contact** | Belongs to exactly one audience. Status: subscribed, unsubscribed, cleaned, transactional, archived, pending. |
| **Audience Field** (merge field) | Typed custom field defined at the audience level: text, number, date, address, phone, image, dropdown, radio. Each gets a merge tag like `*\|FNAME\|*`. |
| **Tag** | Free-form internal label, n:m with contacts. Built for *internal* organization; contrast with groups, which subscribers self-select. |
| **Group** | Subscriber-facing categories users self-pick on signup forms (e.g. "interests"). |
| **Segment** | Saved query over fields, tags, groups, behavior, ecommerce, location, predictive, and dates. Combines up to 5 conditions (Standard) with `match=all\|any\|none`. Can be **static**, **dynamic** (auto-refresh on send), or **saved**. |

Sources: [Mailchimp segmenting options](https://mailchimp.com/help/all-the-segmenting-options/),
[Tags help](https://mailchimp.com/help/getting-started-tags/),
[Segments API ref](https://mailchimp.com/developer/marketing/api/list-segments/).

### 2.2 MailerLite

| Concept | Notes |
|---|---|
| **Subscriber** | Single list at account level (no per-list duplication). Status: active, unsubscribed, unconfirmed, bounced, junk. |
| **Field** | 9 default fields (name, last name, email, phone, company, country, state, city, zip) + unlimited typed custom fields. |
| **Group** | Manual / API-assigned bucket. Static — doesn't recompute. Like Mailchimp's "tag" in spirit. |
| **Segment** | Dynamic, rule-based, auto-updating. Conditions over groups, fields, signup date, timezone, prior campaigns, automation activity, email actions. |

Sources: [Subscribers API](https://developers.mailerlite.com/docs/subscribers.html),
[Groups vs segments](https://www.mailerlite.com/help/what-is-the-difference-between-groups-and-segments),
[Segmentation features](https://www.mailerlite.com/features/segmentation),
[Custom fields](https://www.mailerlite.com/help/how-to-create-and-use-custom-fields).

### 2.3 Kit (ConvertKit)

| Concept | Notes |
|---|---|
| **Subscriber** | One global list per account. Single source of truth. Notable simplification compared to Mailchimp's multi-audience model. |
| **Custom Field** | Typed key-value attached to subscriber. |
| **Tag** | Static label, manually applied or rule-applied. **Tags are the primary organizing primitive in Kit** — there's no "list" abstraction. Subscribers can have unlimited tags. Tag-add is also an automation event. |
| **Segment** | Dynamic, fluid filter built from tags + form + location + signup date. Auto-updates. Notably, segments **cannot** be used to trigger automations or as conditional paths — only tags can. Use tags for triggers, segments for sending. |
| **Form** | Top-level entity. Each form is a subscription source; "subscribed via form X" is a queryable condition. |
| **Sequence** | Linear drip series — older Kit primitive. Can be embedded inside Visual Automations. |

Sources: [Tags vs segments](https://help.kit.com/en/articles/4257108-tags-and-segments-in-kit-and-when-to-use-which),
[Visual Automations actions](https://help.kit.com/en/articles/2502537-visual-automations-actions).

### 2.4 Synthesis: how `mailing-list-cli` should model this

A pragmatic model that covers all three:

```
Contact { id, email, status, fields: {key: value}, tags: [tag_id], created_at, ... }
List    { id, name }                            # multi-list optional, tag-style by default
ListMembership(contact_id, list_id, joined_at)
Tag     { id, name }
Field   { key, type: text|number|date|bool|select, options? }
Segment { id, name, filter: SegmentExpr, materialized?: bool }

SegmentExpr =
    Match { mode: all|any|none, conditions: [Condition] }
Condition =
    | FieldOp { key, op, value }
    | TagOp { mode: has|missing, tag_id }
    | ListOp { mode: in|out, list_id }
    | EngagementOp { event: opened|clicked|sent|bounced, scope, since, count? }
    | DateOp { key, op, value | relative }
```

This subsumes Mailchimp's audience, MailerLite's groups, and Kit's tag-only world by
treating List as an optional explicit bucket and Tag as the primary lightweight label.

---

## 3. Segmentation engines

| | Mailchimp | MailerLite | Kit |
|---|---|---|---|
| **Modes** | all / any / none | all / any | all / any |
| **Max conditions** | 5 (Standard) / unlimited (Advanced/Premium) | unlimited | unlimited |
| **Field types** | text, number, date, address, dropdown, radio | text, number, date, boolean | text, number, date |
| **Engagement filters** | opens/clicks per campaign, signup source, automation start/complete | campaign opened/clicked, automation activity, email actions | tag-presence (proxy), form, signup date, location |
| **Date math** | relative + absolute, anniversary, "X days ago" | relative + absolute | signup date relative |
| **E-commerce** | rich (purchase frequency, value, category, predicted CLV) | basic (e-com plugin) | basic, via integrations |
| **Predictive (RFM-ish)** | yes — predicted CLV, churn risk, gender, age | no | no (engagement score on Pro only) |
| **Re-evaluation** | dynamic at send time | live | live |

Mailchimp's segment-condition JSON shape is a `{conditions: [...], match: "all"\|"any"\|"none"}`
where each condition has a discriminated `condition_type` plus `op`, `value`, and field-typed
extras. MailerLite and Kit expose simpler typed filter objects via REST. Sources:
[Mailchimp segments API](https://mailchimp.com/developer/marketing/api/list-segments/),
[Segment logic explained](https://mailchimp.com/help/understanding-advanced-segmentation-logic/).

**Implication for the CLI**: a single canonical filter AST (rust enum) that serializes to
both a query plan against Resend's contact list and a stored JSON spec is the right call.
Match modes = `All`, `Any`, `None`. Conditions are typed.

---

## 4. Automation / workflow models

### 4.1 Mailchimp
**Customer Journeys** (current) and legacy **Automations**. Visual builder. Triggers:
signup, tag add/remove, date, ecommerce purchase, custom event, automation completion.
Actions: send email, wait, branch (yes/no), update field, add/remove tag, send to other
journey, send SMS. Includes goal/exit conditions.

### 4.2 MailerLite
**Workflows**: trigger → step list. Up to 3 entry triggers per automation, up to 100 steps.
Triggers: joins group, completes form, clicks link, anniversary/date, custom field update,
buys product, abandoned cart. Step types: **Email**, **Delay**, **Condition** (if/else
over campaign activity, segment, group, custom field), **A/B Split**, **Action** (update
field, copy to group, move to group, remove from group, unsub, webhook). Available on free
plan. Sources: [Automation features](https://www.mailerlite.com/features/automation),
[Automation steps](https://www.mailerlite.com/help/how-to-use-automation-steps).

### 4.3 Kit
**Visual Automations** (DAG) sit on top of older linear **Sequences**.
Entry events: subscribes to form, completes purchase, tag added, custom field updated,
joins segment. Actions: subscribe to sequence, add tag, remove tag, set custom field,
send email, webhook, wait. Conditions: tag-based branching. Power-user pattern: tag +
visual automation > sequence alone.
Source: [Visual Automations actions](https://help.kit.com/en/articles/2502537-visual-automations-actions).

**Common DAG primitives** for the CLI:
```
Trigger:    on_subscribe | on_tag_added | on_field_change | on_date | on_event | on_purchase
Step:       send_email | wait(duration|until_date) | branch(condition) |
            add_tag | remove_tag | set_field | http_post | end
Goal/Exit:  optional, removes contact from automation when met
```

---

## 5. A/B testing

| | Mailchimp | MailerLite | Kit |
|---|---|---|---|
| **What can vary** | Subject, From name, Content, Send time (multivariate: pick 3 of 4) | Subject, From name, Content | Subject only (Creator Pro only) |
| **Variations** | up to 8 in multivariate | 2 (A/B) | 2 |
| **Sample size** | 10% min, recommends 5,000/variant | typically 20% of list, min ~1000 | percentage configurable |
| **Winner picked by** | open rate, click rate, total revenue, or manual | open rate or click rate (clicks recommended post-Apple MPP), or manual | open rate |
| **Auto-send winner** | yes, at chosen interval | yes, after 4–48h window | yes |
| **Winner test duration** | 4h–14d | 4h+ recommended (24–48h ideal) | configurable |

Sources: [Mailchimp A/B](https://mailchimp.com/help/about-ab-tests/),
[Mailchimp multivariate](https://mailchimp.com/help/about-multivariate-tests/),
[MailerLite A/B](https://www.mailerlite.com/features/ab-testing),
[Kit Creator Pro features](https://creatoregg.com/kit-review).

**For the CLI**: model an A/B test as `Campaign { variants: [V], winner_pick: by_opens \|
by_clicks \| manual, sample_size: percent | absolute, decision_after: duration }`.
Ship subject + content as the v1 variants, defer multivariate.

---

## 6. Personalization syntax

| Platform | Basic merge | Conditional |
|---|---|---|
| **Mailchimp** | `*\|FNAME\|*`, `*\|EMAIL\|*` | `*\|IF:AGE >= 21\|*…*\|ELSEIF:…\|*…*\|ELSE:\|*…*\|END:IF\|*` (IFNOT also supported). Merge tags case-sensitive. |
| **MailerLite** | `{$name}`, `{$email}`, `{$industry}` for custom fields. System tags: `{$unsubscribe}`, `{$preferences}`, `{$forward}`, `{$url}` | "Conditional content blocks" via UI; per-segment dynamic content blocks at the editor level. |
| **Kit** | Liquid: `{{ subscriber.first_name }}`, `{{ subscriber.email_address }}`, `{{ subscriber.custom_field_name }}` | Liquid `{% if subscriber.tags contains "Purchased course" %} … {% elsif … %} … {% else %} … {% endif %}`. Filters: `\| capitalize`, `\| default: "there"`, `\| strip`. |

Sources: [Mailchimp merge cheat sheet](https://mailchimp.com/help/all-the-merge-tags-cheat-sheet/),
[Conditional merge tags](https://mailchimp.com/help/use-conditional-merge-tag-blocks/),
[MailerLite variables](https://www.mailerlite.com/help/how-to-use-variables-in-mailerlite),
[Kit Liquid basics](https://help.kit.com/en/articles/2502633-basic-email-personalization-with-liquid-faqs),
[Kit Liquid advanced](https://help.kit.com/en/articles/2502501-advanced-email-personalization-with-liquid).

**Recommendation**: adopt **Liquid** as the merge syntax. It's the most expressive of the
three, has open-source implementations in multiple languages (including Rust crates like
`liquid`), and matches what creator-tier customers already know from Kit and Shopify.

---

## 7. Bounce / complaint / unsubscribe handling

### Mailchimp
- **Hard bounce** → contact moved to "Cleaned" status immediately and excluded from
  future sends.
- **Soft bounce** → 7 strikes (no prior activity) or up to 15 (with prior activity)
  before escalation to cleaned.
- **Spam complaint** (via ISP feedback loops) → moved out of active audience into
  "Abuse complaints" pile, never sent to again.
- **Unsubscribe** → kept visible (so user can still target via Postcards/SMS/retargeting).
- All of the above surface in audience UI, exportable, queryable via API.

Source: [Soft vs hard bounces](https://mailchimp.com/help/soft-vs-hard-bounces/),
[About abuse complaints](https://mailchimp.com/help/about-abuse-complaints/),
[About bounces](https://mailchimp.com/help/about-bounces/).

### MailerLite
- Status enum: `active | unsubscribed | unconfirmed | bounced | junk`. Bounced and junk
  are auto-set, no manual sending allowed.
- "Forget" endpoint deletes within 30 days for GDPR.
- Strict bounce-rate enforcement: high bounce rate triggers account suspension (a top
  user complaint).
- One-click List-Unsubscribe header **on by default**.
Source: [Subscribers API](https://developers.mailerlite.com/docs/subscribers.html),
[List-unsubscribe guide](https://www.mailerlite.com/help/a-simple-guide-list-unsubscribe-header-and-one-click-unsubscribe).

### Kit
- Subscriber states: active, cancelled, bounced, complained.
- Bounces and complaints auto-remove from sending pool.
- Cold subscriber re-engagement helper built in (Creator Pro).

**For the CLI** — every contact needs a `status` enum that mirrors this, plus a
`suppressed_reason` field (`hard_bounce | soft_bounce_threshold | complaint |
manual_unsub | one_click_unsub | gdpr_forget`). Sends MUST hard-skip non-`active`.

---

## 8. Compliance features

| Feature | Mailchimp | MailerLite | Kit |
|---|---|---|---|
| **Double opt-in** | toggle per audience | toggle per form | toggle per form |
| **GDPR fields/consent log** | yes, with audit trail | yes; "GDPR tools" with right-to-portability + right-to-be-forgotten ("Forget" endpoint) | yes |
| **CAN-SPAM footer** | auto-injected (physical address) | auto-injected | auto-injected |
| **Preference center** | yes | yes (configurable groups) | yes |
| **List-Unsubscribe (RFC 8058 one-click)** | yes (default) | yes (default) | yes (default) |
| **Per-account legal address** | required | required | required |

Source: [MailerLite GDPR](https://help.mailerlite.com/article/show/59543-gdpr-tools),
[Mailchimp deliverability/compliance](https://mailchimp.com/help/about-bounces/),
[List-unsubscribe rules 2024](https://www.mailerlite.com/blog/new-requirements-from-google-and-yahoo).

**CLI must auto-inject** (not optional): physical address, unsubscribe link, and the
RFC 8058 `List-Unsubscribe` + `List-Unsubscribe-Post` headers. These are table stakes.

---

## 9. Templates

| | Mailchimp | MailerLite | Kit |
|---|---|---|---|
| **Drag & drop** | "New Builder" + Classic Builder | Drag & drop editor | Limited block editor |
| **Rich text** | yes | yes | yes (default) |
| **Raw HTML import** | yes (Classic Builder + custom HTML editor) | yes (Custom HTML editor) | **no** — major Kit weakness |
| **Block library / saved blocks** | content blocks + repeatable blocks | block templates (save/reuse) | minimal |
| **Responsive by default** | yes | yes | yes |
| **Code editor** | full HTML in Classic | source code view | none |
| **Template count out-of-box** | hundreds | many | only ~8 starter templates |

Sources: [Mailchimp builders](https://mailchimp.com/help/design-an-email-new-builder/),
[Mailchimp template code](https://mailchimp.com/help/where-to-edit-template-code/),
[MailerLite editors](https://www.mailerlite.com/features/newsletter-editors),
[Kit weaknesses](https://creatoregg.com/kit-review).

---

## 10. APIs and programmatic surface

### 10.1 Mailchimp Marketing API v3.0
- **Base**: `https://<dc>.api.mailchimp.com/3.0/`
- **Auth**: API key (with embedded `dc`) **or** OAuth 2.0
- **Rate**: 10 simultaneous connections per API key (use Batch endpoint for bulk)
- **Core resources**: `lists` (audiences), `lists/{id}/members`, `lists/{id}/segments`,
  `lists/{id}/tag-search`, `lists/{id}/merge-fields`, `lists/{id}/interest-categories`,
  `campaigns`, `automations`, `templates`, `reports`, `batches`, `webhooks`, `e-commerce`,
  `landing-pages`, `file-manager/files`.
- **Webhooks**: subscribe, unsubscribe, profile, cleaned, upemail, campaign.
Source: [Fundamentals](https://mailchimp.com/developer/marketing/docs/fundamentals/),
[Methods](https://mailchimp.com/developer/marketing/docs/methods-parameters/).

### 10.2 MailerLite API
- **Base**: `https://connect.mailerlite.com/api`
- **Auth**: Bearer token (`Authorization: Bearer XXX`); tokens bound to creator user.
- **Rate**: 120 req/min per API key; batch endpoint for bulk; SDKs in PHP/Go/Node/Python/Ruby.
- **Core resources**: `subscribers` (list/create/update/upsert/forget), `groups`,
  `segments`, `fields`, `campaigns`, `automations`, `forms`, `webhooks`, `batch`, `timezones`.
Source: [API getting started](https://developers.mailerlite.com/docs/),
[Subscribers](https://developers.mailerlite.com/docs/subscribers.html).

### 10.3 Kit API v4
- **Base**: `https://api.kit.com/v4/...`
- **Auth**: OAuth 2.0 (apps) or API key (personal); supports cursor-based pagination,
  bulk requests, async processing.
- **Rate**: 600 req/60s on OAuth, 120 req/60s on API key.
- **Core resources**: `subscribers`, `tags`, `tags/{id}/subscribers`, `custom_fields`,
  `forms`, `sequences`, `broadcasts`, `webhooks`, `account`. Tag unsubscribe path moved
  from `POST /v3/tags/:id/unsubscribe` to `DELETE /v4/tags/:tag_id/subscribers/:id`.
Source: [Kit API v4](https://developers.kit.com/v4),
[Authentication](https://developers.kit.com/api-reference/authentication).

### 10.4 Synthesis for `mailing-list-cli`
All three converge on the same vocabulary:

```
contacts | lists | segments | tags | fields | forms | broadcasts | automations |
templates | reports | webhooks | batches
```

This is the natural top-level command surface for the CLI:

```
mailing-list-cli contact   {list,get,create,update,delete,upsert,bulk-import}
mailing-list-cli list      {ls,create,delete,merge}
mailing-list-cli tag       {ls,create,apply,remove}
mailing-list-cli field     {ls,create,update,delete}
mailing-list-cli segment   {ls,create,test,materialize}
mailing-list-cli form      {ls,create,...}
mailing-list-cli broadcast {draft,schedule,send,test,abtest}
mailing-list-cli automation{ls,create,enable,disable,run-once}
mailing-list-cli template  {ls,create,update,render}
mailing-list-cli report    {broadcast,contact,segment,deliverability}
mailing-list-cli webhook   {ls,create,delete}
```

---

## 11. Biggest weaknesses (where the new CLI can win)

### Mailchimp
- **Pricing**: aggressive Intuit-era price hikes. Free plan dropped from 2,000 → 250
  contacts; automations stripped from free tier mid-2025. Capterra reviews flag
  "expensive" 81 times.
- **Pricing model**: bills per *contact* including unsubs and inactives, with auto-overage.
- **Deliverability slipping**: ~85% delivered, ~75% to primary inbox in 2026 tests —
  not great for the price.
- **Multi-audience confusion**: hard list separation contradicts modern unified-contact
  best practice.

Source: [Mailchimp pricing complaints](https://www.retainful.com/blog/mailchimp-pricing),
[Mailchimp deliverability 2026](https://www.emailtooltester.com/en/blog/mailchimp-deliverability/).

### MailerLite
- **Onboarding friction**: strict approval / verification (60% of approval-related
  reviews are negative).
- **Sudden suspensions**: tight bounce-rate enforcement triggers account termination.
- **Limited reporting depth**: no deliverability dashboard, can't filter Apple MPP opens
  or bot clicks, no custom reports.
- **No migration assistance** ("self-serve").
Source: [MailerLite review](https://www.emailtooltester.com/en/reviews/mailerlite/).

### Kit (ConvertKit)
- **No raw HTML in editor**, only ~8 starter templates, weak design flexibility — text
  newsletters are "the way".
- **A/B testing locked behind Creator Pro and only tests subject lines**.
- **Email-only**, no SMS/push, limited multi-channel.
- **Customization gaps**: can't reorder/duplicate sections, no preview, image cropping issues.
Source: [Kit review pros/cons](https://creatoregg.com/kit-review),
[Bloggingwizard Kit review](https://bloggingwizard.com/convertkit-review-and-tutorial/).

### Where `mailing-list-cli` wins
1. **Flat per-send pricing via Resend** — no contact-tier extortion.
2. **Single global subscriber list** like Kit, but with optional explicit lists like
   Mailchimp — best of both.
3. **JSON-first, scriptable** — no UI lock-in, agents drive everything.
4. **Raw HTML and Liquid templates** at the same level as block builder.
5. **Liquid-first personalization** — most expressive, open-source render path.
6. **Auto-suppression that's loud and visible** in CLI output (`status` field on every
   contact, not buried in a UI tab).
7. **One-click List-Unsubscribe + footer auto-injection** — non-skippable.
8. **`agent-info` self-description** — agents can introspect commands with no docs scraping.

---

## Sources

- [Mailchimp segmenting options](https://mailchimp.com/help/all-the-segmenting-options/)
- [Mailchimp tags & custom fields](https://mailchimp.com/features/tags/)
- [Mailchimp segments API reference](https://mailchimp.com/developer/marketing/api/list-segments/)
- [Mailchimp advanced segmentation logic](https://mailchimp.com/help/understanding-advanced-segmentation-logic/)
- [Mailchimp merge tags cheat sheet](https://mailchimp.com/help/all-the-merge-tags-cheat-sheet/)
- [Mailchimp conditional merge tags](https://mailchimp.com/help/use-conditional-merge-tag-blocks/)
- [Mailchimp A/B tests](https://mailchimp.com/help/about-ab-tests/)
- [Mailchimp multivariate tests](https://mailchimp.com/help/about-multivariate-tests/)
- [Mailchimp soft vs hard bounces](https://mailchimp.com/help/soft-vs-hard-bounces/)
- [Mailchimp abuse complaints](https://mailchimp.com/help/about-abuse-complaints/)
- [Mailchimp Marketing API fundamentals](https://mailchimp.com/developer/marketing/docs/fundamentals/)
- [Mailchimp builders](https://mailchimp.com/help/design-an-email-new-builder/)
- [Mailchimp pricing complaints](https://www.retainful.com/blog/mailchimp-pricing)
- [Mailchimp deliverability 2026](https://www.emailtooltester.com/en/blog/mailchimp-deliverability/)
- [MailerLite API getting started](https://developers.mailerlite.com/docs/)
- [MailerLite Subscribers API](https://developers.mailerlite.com/docs/subscribers.html)
- [MailerLite groups vs segments](https://www.mailerlite.com/help/what-is-the-difference-between-groups-and-segments)
- [MailerLite segmentation features](https://www.mailerlite.com/features/segmentation)
- [MailerLite custom fields](https://www.mailerlite.com/help/how-to-create-and-use-custom-fields)
- [MailerLite automations](https://www.mailerlite.com/features/automation)
- [MailerLite automation steps](https://www.mailerlite.com/help/how-to-use-automation-steps)
- [MailerLite A/B testing](https://www.mailerlite.com/features/ab-testing)
- [MailerLite variables and merge tags](https://www.mailerlite.com/help/how-to-use-variables-in-mailerlite)
- [MailerLite editors](https://www.mailerlite.com/features/newsletter-editors)
- [MailerLite GDPR tools](https://help.mailerlite.com/article/show/59543-gdpr-tools)
- [MailerLite list-unsubscribe header](https://www.mailerlite.com/help/a-simple-guide-list-unsubscribe-header-and-one-click-unsubscribe)
- [MailerLite review weaknesses](https://www.emailtooltester.com/en/reviews/mailerlite/)
- [Kit API v4 overview](https://developers.kit.com/v4)
- [Kit API authentication](https://developers.kit.com/api-reference/authentication)
- [Kit tags and segments](https://help.kit.com/en/articles/4257108-tags-and-segments-in-kit-and-when-to-use-which)
- [Kit Visual Automations actions](https://help.kit.com/en/articles/2502537-visual-automations-actions)
- [Kit Liquid basics](https://help.kit.com/en/articles/2502633-basic-email-personalization-with-liquid-faqs)
- [Kit Liquid advanced](https://help.kit.com/en/articles/2502501-advanced-email-personalization-with-liquid)
- [Kit pros/cons review](https://creatoregg.com/kit-review)
- [Bloggingwizard Kit review](https://bloggingwizard.com/convertkit-review-and-tutorial/)
- [Google/Yahoo 2024 sender requirements](https://www.mailerlite.com/blog/new-requirements-from-google-and-yahoo)
