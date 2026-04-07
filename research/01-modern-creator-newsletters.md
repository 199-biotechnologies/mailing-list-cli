# Modern Creator Newsletters: Beehiiv, Buttondown, Substack

Research date: 2026-04-07. Scope: feature reality (not marketing copy) for creators with 5k–100k subscribers, framed as input for `mailing-list-cli`, a Resend-backed agent-friendly Rust CLI.

## TL;DR — the 80/20 feature set every operator needs

Across all three platforms, daily life for a 5k–100k-list creator reduces to roughly **eight surfaces**. Anything outside this list is either a differentiator or rarely-touched cruft.

1. **Compose + send a campaign** (subject, from-name, body, scheduled time). Markdown is acceptable; HTML is required as an escape hatch.
2. **Subscriber CRUD + bulk import** (CSV or API; with tag/source on import).
3. **Tags/segments** at minimum: by signup source, by tag, by activity (opened/clicked X in last N days).
4. **Send to a segment** (not just "everyone").
5. **A/B test the subject line** (2–4 variants, time-bounded, auto-pick winner). This is now table stakes after Substack added it in May 2025.
6. **Per-campaign analytics**: opens, clicks, click-per-link, unsubscribes, bounces.
7. **Bounce + complaint handling** (auto-suppress hard bounces, surface soft-bounce trend, suppression list export).
8. **Welcome automation / drip** (≥2-step sequence triggered by subscribe-event). Anything more complex is a differentiator.

If `mailing-list-cli` ships these eight cleanly, it covers what the *median* operator on Beehiiv and Buttondown actually does day-to-day — and beats Substack outright.

---

## Platform-by-platform

### 1. Beehiiv — the "newsletter as growth product" stack

**What real users do with it:** Send broadcasts, run welcome automations, A/B test subjects, run referral programs, monetize via the ad network. Reviewers consistently call out segmentation, analytics, and automations as best-in-class for the creator segment.[^beehiiv-emailaudience][^beehiiv-emailtooltester]

**Send flow.** Compose post → set subject → optionally enable A/B (up to 4 subject variants, 5–240 min test window, auto-send winner) → choose audience (everyone / segment) → schedule or send.[^beehiiv-abtest]

**Segmentation.** Combine criteria with AND/OR (signup source, tags, activity, open/click history, custom data). Behavioral segments are the differentiator vs Substack.[^beehiiv-emailtooltester]

**Automations.** Visual journey builder with triggers (subscribe, tag added, segment join), actions (send email, add tag, wait, branch), time delays, and percentage splits for A/B-in-flow / holdouts.[^beehiiv-automations] Critics note workflows are basic vs Kit/ActiveCampaign — fine for welcome + re-engagement, weak for behavioral funnels.[^beehiiv-marketermilk]

**Analytics shipped per post.** Recipients, open rate, CTR, unsubscribe rate, per-link click breakdown, "verified unique clicks" (filters bot prefetch), 3D analytics (post / subscribers / clicks reports).[^beehiiv-postdash][^beehiiv-clicks]

**Templates.** Block editor (sections, images, buttons, polls, ads, web embeds). Custom HTML available but the editor is the daily driver. Reviewers call the editor "the weaker spot" for visual-heavy newsletters.[^beehiiv-marketermilk]

**Scale features.** Dedicated IP available (Beehiiv recommends only past ~50k/month).[^beehiiv-dedicatedip] Automated re-engagement / list hygiene flow built in for inactive cleanup.[^beehiiv-listhygiene] Bounce/complaint handling is automatic (powered by SendGrid Engagement Quality Score under the hood).[^beehiiv-twilio]

**Deliverability reporting.** Per-post bounce/complaint metrics + general dashboards. No granular reputation graph by ISP — that lives in Beehiiv's infra, not in the user's UI.

**Biggest weakness (consensus).** (1) Steep pricing jump (free → $49 Scale, no middle tier).[^beehiiv-pricing] (2) Email design editor is shallow vs Mailchimp/Kit. (3) Custom-field support and external integrations limited — hard to wire into CRM/e-commerce.[^beehiiv-marketermilk] (4) Automation workflows feel "basic-plus" once you push past welcome flows.[^beehiiv-marketermilk]

---

### 2. Buttondown — the developer's markdown newsletter

**What real users do with it:** Markdown-first writers and developers managing small-to-mid lists, often with custom integrations. Solo-founder operation (Justin Duke). Consistent praise for the API and markdown rendering.[^buttondown-woodpecker][^buttondown-sequenzy]

**Send flow.** Write in Markdown (or rich text — both supported). Set subject, schedule. Code blocks render with syntax highlighting in archive *and* monospace in email — best-in-class for technical writers.[^buttondown-sequenzy]

**Segmentation.** Tags + segments + "send to subset" by criteria. Good enough for the typical tech newsletter; not a behavioral powerhouse.[^buttondown-softwareadvice]

**Automations.** Email sequences / drip + automated sends. Tags as segmentation primitives. Fewer canned workflows than Beehiiv but the API + webhooks let you build them.[^buttondown-softwareadvice]

**A/B testing.** Subject-line A/B + accessibility tests built in.[^buttondown-softwareadvice]

**Analytics.** Standard set: opens, clicks, unsubscribes, bounces, click-per-link. Privacy-conscious — no fingerprinting.[^buttondown-woodpecker]

**Templates.** Markdown is the canonical format; raw HTML supported. No block editor. This *is* the differentiator (not the weakness) for the target audience.[^buttondown-features]

**API surface (the differentiator vs both others).** REST API covers subscribers, emails, tags, scheduling, automations, drafts, RSS-to-email, webhooks. Built API-first; reviewers cite it as the primary reason they choose Buttondown over Substack.[^buttondown-softwareworld] **Most relevant precedent for `mailing-list-cli`.**

**Scale features.** Buttondown is the smallest of the three operationally; users past ~25k typically migrate or stay because they value the markdown/API flow over scale features. No dedicated IP / warmup flow advertised. Suppression lists exist via API.

**Biggest weakness (consensus).** Smallest brand, no monetization stack, no referral network, sparse template gallery, no growth-tooling (no ad network, no recommendations marketplace). Pricing scales linearly so list of 50k+ is expensive.[^buttondown-newsletterco]

---

### 3. Substack — the reach + monetization mass market

**What real users do with it:** Write posts, collect paid subscribers, leverage Notes + recommendations + Substack's discovery surface for free distribution. The platform is the *audience*, not the toolset.[^substack-pubstack]

**Send flow.** Compose post (rich text editor; limited HTML), choose free / paid / founder tier audience, schedule or publish. As of May 2025, optionally enable Title Test (≤4 title variants, default 50% audience, 1-hour test, auto-send winner — minimum 200 subscribers).[^substack-titletest][^substack-abnews]

**Segmentation.** Effectively: free vs paid vs founder. That's it. No tags, no behavioral segments, no signup-source filters.[^substack-pros][^substack-techradar] Substack itself describes "segmentation" as primarily the free/paid split.[^substack-tella]

**Automations / drip.** None in any meaningful sense. No welcome sequences, no behavior triggers, no onboarding flows.[^substack-pros][^substack-outthink]

**Analytics shipped per post.** Open rate, click rate, unsubscribes, growth chart, top posts, gross annualized revenue (the centerpiece). Per-link clicks limited. No device/geo breakdown comparable to ESPs.[^substack-metrics]

**Templates.** None. One house style, one editor. By design.

**Scale features.** Substack handles infrastructure (no IP warmup concept exposed, no suppression import). Bounces and unsubscribes auto-handled.

**Biggest weakness (consensus).** (1) **10% revenue cut on paid subs** is the dominant complaint as creators scale.[^substack-multidots] (2) No automation, no segmentation, no integrations with CRM/e-commerce.[^substack-pros] (3) You don't own the brand surface or design. (4) A/B testing only landed in May 2025 and is still title-only.[^substack-abnews]

---

## Comparison matrix (what's actually shippable today)

| Capability | Beehiiv | Buttondown | Substack |
|---|---|---|---|
| Markdown source | partial | **canonical** | no (rich text) |
| HTML escape hatch | yes | yes | limited |
| Block editor | yes | no | minimal |
| Tags + segments | strong (AND/OR + behavior) | yes (basic) | free/paid only |
| Welcome / drip automation | yes (visual builder) | yes (API-driven) | **none** |
| A/B testing | subject + send time + flow split | subject + accessibility | subject only (May 2025+) |
| Per-link click report | yes | yes | basic |
| Bounce auto-suppress | yes | yes | yes |
| List-cleaning automation | yes (re-engagement flow) | manual via API | none |
| Dedicated IP | yes (≥50k/mo) | no | n/a (managed) |
| API-first | secondary | **primary** | minimal |
| Geo / device breakdown | limited | limited | limited |
| Revenue cut | 0% | 0% | 10% |

---

## Implications for `mailing-list-cli`

The **non-negotiable agent-CLI surface** is the eight items in the TL;DR. Specifically map:

- `compose` / `send` / `schedule` — accept markdown or html, subject, from, audience selector
- `subscribers add|import|list|tag|untag|delete` with bulk import semantics (CSV + JSONL)
- `segments create|list` supporting AND/OR over tag, signup_source, activity windows
- `ab-test` flag on send: `--ab-subjects`, `--ab-window=60m`, `--ab-winner-metric=open|click`
- `analytics campaign <id>` returning structured JSON: opens, clicks, ctr, click-per-link, unsubs, bounces (hard/soft), complaint
- `bounces` and `complaints` as first-class commands (since Resend exposes both via webhooks)
- `automation` minimum: trigger=on_subscribe, action=send_email_after(N), branch=tag_added — keep it lean
- `suppression list|import|export` (Substack ignores this; both ESPs treat it as table stakes)

**Differentiation opportunities** vs the three platforms above:
- **Be the markdown-first agent CLI Buttondown wishes it had a CLI for** — this is the closest precedent and the one to beat on UX.
- **Structured JSON output** is unique. None of the three offer scriptable terminal output; Buttondown's API is the closest but still requires you to write a client.
- **Diff-able campaign artifacts** (campaign-as-a-file) is something none of them do — appeals to the Git-native dev audience.
- **Don't try to compete on monetization, ad network, or block editor** — not the wedge. The wedge is "I run a real list from the terminal in 50 lines of bash."

**Things to deliberately not build at v1** (low ROI):
- Dedicated-IP management (Resend handles this above the API line; pass through later if needed)
- IP warmup automation
- Block editor / WYSIWYG (markdown→html via a templating step is enough)
- Referral / monetization
- Native unsubscribe page (Resend provides one; just expose the URL)

---

## Sources

[^beehiiv-emailaudience]: [Beehiiv Review 2026: My experience after 90 days — emailaudience.com](https://www.emailaudience.com/beehiiv-review/)
[^beehiiv-emailtooltester]: [Beehiiv Review (2026): Is It The Best Tool for Newsletter Growth? — emailtooltester.com](https://www.emailtooltester.com/en/reviews/beehiiv/)
[^beehiiv-abtest]: [Email A/B Testing Tool for Newsletters — beehiiv.com](https://www.beehiiv.com/features/ab-testing) and [Creating and using A/B tests in your beehiiv posts](https://www.beehiiv.com/support/article/9479415454615-creating-and-using-ab-tests-in-your-beehiiv-posts)
[^beehiiv-automations]: [Using Automations: Overview of triggers and actions — beehiiv KB](https://www.beehiiv.com/support/article/13080928484887-using-automations-overview-of-triggers-and-actions)
[^beehiiv-marketermilk]: [Beehiiv review: My honest thoughts after 2 years in use — marketermilk.com](https://www.marketermilk.com/blog/beehiiv-review)
[^beehiiv-postdash]: [Post analytics dashboard walkthrough — beehiiv KB](https://www.beehiiv.com/support/article/24862562076183-where-to-view-post-analytics)
[^beehiiv-clicks]: [Understanding your Clicks Report — beehiiv KB](https://www.beehiiv.com/support/article/29771239226391-understanding-your-clicks-report)
[^beehiiv-dedicatedip]: [When a dedicated IP address is needed for beehiiv — beehiiv KB](https://www.beehiiv.com/support/article/13092212986647-when-a-dedicated-ip-address-is-needed-for-beehiiv)
[^beehiiv-listhygiene]: [List Hygiene - The Most Underrated Way to Increase Your Open Rates — beehiiv blog](https://www.beehiiv.com/blog/list-hygiene-the-most-underrated-way-to-increase-your-open-rates)
[^beehiiv-twilio]: [beehiiv improves email deliverability with SendGrid's Engagement Quality Score — twilio.com](https://customers.twilio.com/en-us/beehiiv)
[^beehiiv-pricing]: [Beehiiv Pricing: All Plans & True Costs Compared (2026) — sender.net](https://www.sender.net/reviews/beehiiv/pricing/)
[^buttondown-woodpecker]: [Buttondown for Email Newsletters: 2026 Review — woodpecker.co](https://woodpecker.co/blog/buttondown/)
[^buttondown-sequenzy]: [7 Best Email Tools for Developer Newsletters (2026) — sequenzy.com](https://www.sequenzy.com/blog/best-email-tools-for-developer-newsletters)
[^buttondown-softwareadvice]: [Buttondown Software Reviews, Demo & Pricing — softwareadvice.com](https://www.softwareadvice.com/email-marketing/buttondown-profile/)
[^buttondown-features]: [Write in Markdown (And then some) — buttondown.com](https://buttondown.com/features/markdown)
[^buttondown-softwareworld]: [Buttondown Reviews Jan 2026: Pricing & Features — softwareworld.co](https://www.softwareworld.co/software/buttondown-reviews/)
[^buttondown-newsletterco]: [Buttondown Review 2026: Pricing, Features, Pros & Cons — newsletter.co](https://newsletter.co/buttondown-review/)
[^substack-pubstack]: [What are the Downsides of Substack? — pubstacksuccess.substack.com](https://pubstacksuccess.substack.com/p/what-are-the-downsides-of-substack)
[^substack-titletest]: [How do I test different titles for email newsletters on Substack? — Substack support](https://support.substack.com/hc/en-us/articles/36026518014100-How-do-I-test-different-titles-for-email-newsletters-on-Substack)
[^substack-abnews]: [New on Substack: A/B testing for headlines — on.substack.com](https://on.substack.com/p/new-on-substack-ab-testing-for-headlines)
[^substack-pros]: [Substack pros and cons — minimadesigns.com](https://minimadesigns.com/substack-pros-and-cons)
[^substack-techradar]: [Substack review 2025 — techradar.com](https://www.techradar.com/pro/website-building/substack-review)
[^substack-tella]: [Subscriber Segmentation Definition - Substack Explained — tella.com](https://www.tella.com/definition/subscriber-segmentation)
[^substack-outthink]: [Substack vs. Email Marketing Platforms: A Practitioner's Breakdown — outthink.co](https://outthink.co/substack-vs-email-marketing-platforms-a-practitioners-breakdown/)
[^substack-metrics]: [A guide to Substack metrics — Substack support](https://support.substack.com/hc/en-us/articles/5320347155860-A-guide-to-Substack-metrics)
[^substack-multidots]: [7 Powerful Substack Alternatives for Serious Creators — multidots.com](https://www.multidots.com/blog/substack-alternatives/)
