# Deliverability & Compliance Brief — Operating a Mailing List at 10k–500k

> Research date: 2026-04-07. Scope: what `mailing-list-cli` MUST expose so a single operator on Resend can run a 10k–500k list without burning sender reputation or violating law. Opinionated, citation-rich, no ceremony.

---

## TL;DR — The Five Things That Will Wreck You at Scale

1. **You don't have a global suppression list, so an unsubscribe in Campaign A still gets emailed by Campaign B.** Result: spam complaints climb past 0.3%, Gmail blocks you.
2. **You don't auto-suppress hard bounces immediately.** Two more sends to a permanently invalid mailbox = a spam-trap-style signal to Gmail/Yahoo. Reputation crater.
3. **You don't implement RFC 8058 one-click unsubscribe.** As of June 2024, Gmail will not deliver bulk mail without `List-Unsubscribe` + `List-Unsubscribe-Post: List-Unsubscribe=One-Click`. Hard requirement, not best practice.
4. **You ship to a list of "imported contacts" with no proof of consent.** Pristine spam traps fire on first send. Domain reputation tanks in days, not weeks. Hard to recover.
5. **You don't honor unsubscribes within 48 hours.** CAN-SPAM says 10 business days; Gmail/Yahoo 2024 say two days. Whichever is stricter wins. Stale unsubscribes generate complaints.

---

## 1. Bounce Handling

### 1.1 Hard vs soft bounces — definitions and SMTP codes

| Type | SMTP class | Meaning | Action |
|---|---|---|---|
| Hard bounce | **5xx** (permanent) | Server permanently rejects: invalid mailbox, domain doesn't exist, blocked address. Examples: `5.1.1` "bad destination mailbox," `5.1.10` "no such user," `5.7.1` "rejected for policy reasons." | Suppress **immediately and forever**. Never retry. |
| Soft bounce | **4xx** (transient) | Server temporarily rejects: mailbox full (`4.2.2`), greylisting (`4.7.0`), message too large, content-rejected, throttling. | ESP retries 3–5 times over 24–72h. |
| Block | 5xx with policy reason | Often hidden inside 5xx. ISP-side reputation block — not a true address failure, but a sender problem. | Diagnose, do NOT just suppress. |

Sources: [RFC 3463 enhanced status codes](https://reviewmyemails.com/emailalmanac/industry-standards-and-best-practices/bounce-error-handling-standards/rfc-3463-bounce-classification-codes), [SMTP bounce code reference](https://www.suped.com/knowledge/email-deliverability/troubleshooting/what-are-bounce-message-error-codes-and-how-should-i-interpret-them).

### 1.2 How fast must hard bounces be removed?

**Industry consensus: immediately, before the next send to that address.** Hard bounces should be added to the suppression list on receipt of the bounce webhook, not on the next batch run, not on a nightly cron. ISP behavior is unforgiving: a second send to a confirmed-invalid mailbox is treated as a sign you don't manage your list, and will cost you reputation. Postmark explicitly enforces a max bounce rate of 10% and penalizes accounts that breach it ([Postmark spam complaint guide](https://postmarkapp.com/blog/how-to-fix-email-spam-complaints)).

### 1.3 What Resend does automatically vs what the operator must do

**Resend automatically** ([Resend bounce docs](https://resend.com/docs/dashboard/emails/email-bounces)):

- Receives bounce notifications from recipient mail servers.
- Classifies bounces into **Permanent** (subtypes: `General`, `NoEmail`), **Transient** (`General`, `MailboxFull`, `MessageTooLarge`, `ContentRejected`, `AttachmentRejected`), or **Undetermined`.
- Adds permanently-bounced addresses to its internal account-level suppression list ("recipient is on the suppression list because it has a recent history of producing hard bounces"). Future API calls for that address fail-fast.
- Emits `email.bounced` and `email.delivery_delayed` webhook events ([Resend webhooks](https://resend.com/docs/webhooks/introduction)).
- For Audiences/Broadcasts: automatically marks contacts as unsubscribed when they hit the `{{{RESEND_UNSUBSCRIBE_URL}}}` flow ([Resend Audiences](https://resend.com/docs/dashboard/audiences/introduction)).

**The operator (= this CLI) must**:

- Subscribe to the bounce webhook and **mirror Resend's suppression decisions into the local list database**, so future imports / segments / re-broadcasts cannot resurrect a dead address.
- Decide soft-bounce auto-suppression policy. Resend will retry transient bounces but does not (to my reading of the docs) auto-suppress on N-consecutive-soft-bounces.
- Honor permanent bounces across **all audiences in the account**, not just the audience the bounce came from.
- Investigate `ContentRejected` / `AttachmentRejected` — these are content problems, not address problems, and suppressing the user is wrong.

### 1.4 Soft-bounce auto-suppression threshold

Industry consensus: **3–5 consecutive soft bounces over 30–90 days → suppress** ([Suped soft-bounce logic](https://www.suped.com/knowledge/email-deliverability/technical/what-is-the-recommended-soft-bounce-suppression-logic-for-email), [Mystrika bounce guide](https://blog.mystrika.com/handling-email-bounces/)).

Recommended default for the CLI: **5 consecutive soft bounces with no successful delivery in between → auto-suppress**. Make the threshold and time window configurable. Reset the counter on any successful delivery.

Edge case: a mailbox-full (`4.2.2`) loop is the most common soft-bounce-forever pattern. Treat 5+ `MailboxFull` in a row as effectively dead, regardless of overall window.

---

## 2. Spam Complaints (FBL / Abuse Reports)

### 2.1 What counts as a complaint and how it surfaces

A complaint = recipient hit "Mark as spam" / "Junk" / "Report spam" in their mail client. The receiving ISP reports this back to the sender via a **Feedback Loop (FBL)** in [ARF format](https://datatracker.ietf.org/doc/html/rfc5965). Resend (and every modern ESP) ingests these and exposes them as `email.complained` webhook events.

Important: **Gmail does not provide per-message FBL data**. Gmail aggregates complaint rate in [Postmaster Tools](https://postmaster.google.com/) at the domain level. Yahoo, Outlook/Hotmail, AOL, Comcast and most others do provide per-message FBL data.

### 2.2 Acceptable complaint rates

| Source | Threshold | Notes |
|---|---|---|
| **Postmark** | **≤ 0.10%** (1 complaint per 1000 sends) | Account suspension trigger ([Postmark complaint guide](https://postmarkapp.com/blog/how-to-fix-email-spam-complaints)) |
| **Gmail (Postmaster Tools)** | Stay **below 0.10%**; **never exceed 0.30%** | At ≥0.30% Gmail starts diverting to spam folder; sustained 0.30% = ineligible for mitigation ([Gmail sender FAQ](https://support.google.com/a/answer/14229414)) |
| **SendGrid / Mailgun** | Same range; ≤0.10% target, 0.30% danger zone | Industry default |
| **Yahoo** | ≤0.30% | Same as Gmail since Feb 2024 |

Gmail's enforcement detail: bulk senders are **eligible for mitigation when their spam rate remains below 0.30% for 7 consecutive days** ([Suped on Google bulk sender changes](https://www.suped.com/knowledge/email-deliverability/compliance/what-are-the-recent-changes-to-googles-bulk-sender-guidelines)).

### 2.3 What happens if you exceed it

- **0.10% – 0.30%**: degraded inbox placement; some Gmail/Yahoo users start seeing your mail in spam.
- **> 0.30% sustained**: Gmail returns `421 4.7.28` / `550 5.7.1` rejections en masse. Your domain reputation drops to "Bad" in Postmaster Tools and stays there for **weeks**, not hours. Recovery is slow even after the underlying issue is fixed.
- **Postmark** suspends accounts above 0.10% if not corrected.
- Starting **November 2025**, Gmail "ramped up enforcement" — bulk-sender violators get temporary AND permanent rejections, not just spam-foldering ([Security Boulevard 2025 update](https://securityboulevard.com/2025/11/google-and-yahoo-updated-email-authentication-requirements-for-2025/)).

---

## 3. Suppression Lists

### 3.1 Categories that belong on a suppression list

A correct suppression list has **at least these categories**, each with a distinct reason code so they can be filtered, exported, and audited separately:

| Category | Trigger | Permanent? |
|---|---|---|
| `unsubscribed` | User clicked unsubscribe link / used one-click List-Unsubscribe | Permanent (must honor across all campaigns) |
| `hard_bounced` | 5xx SMTP rejection | Permanent |
| `soft_bounced_repeat` | N consecutive soft bounces (default 5) | Permanent |
| `complained` | FBL complaint received | Permanent |
| `manually_blocked` | Operator added (legal request, abuse, GDPR erasure) | Permanent |
| `spam_trap_hit` | Detected by deliverability tool / suspected pristine trap | Permanent + investigate acquisition source |
| `gdpr_erasure` | Article 17 deletion request | Permanent + cryptographic delete (no soft-delete) |
| `role_account` | `info@`, `noreply@`, `postmaster@` etc | Soft suppression — block but warn |

### 3.2 Why a global cross-campaign suppression list is non-negotiable

If suppression is per-campaign or per-audience instead of global:

- A user who unsubscribes from "Newsletter" still gets "Product updates."
- This is the #1 cause of complaint-rate spikes for multi-list operators.
- **CAN-SPAM §316.5** requires opt-out be honored for any "commercial message" by the same sender, not just the specific campaign ([FTC CAN-SPAM compliance guide](https://www.ftc.gov/business-guidance/resources/can-spam-act-compliance-guide-business)).
- **GDPR Article 7(3)** requires withdrawal of consent be as easy as giving it.
- The CLI must enforce: any send call must filter against the global suppression list before composing the recipient batch. Always. No flag to disable.

### 3.3 What goes wrong without one

- Complaint rate creeps from 0.05% to 0.5% in two campaign cycles.
- Domain reputation dies. Even legitimate transactional emails (password resets, receipts) start landing in spam.
- Recovery requires months of low-volume sending to high-engagement addresses. There is no "ask Gmail nicely" button.
- Legal exposure: a single complaint to the ICO (UK) or FTC (US) is enough to trigger an investigation.

---

## 4. Double Opt-In vs Single Opt-In

### 4.1 When is double opt-in legally required vs best practice?

| Jurisdiction | Required? | Source |
|---|---|---|
| **US (CAN-SPAM)** | No, not even single opt-in is required by law for the email itself. CAN-SPAM is opt-out, not opt-in. | [FTC CAN-SPAM guide](https://www.ftc.gov/business-guidance/resources/can-spam-act-compliance-guide-business) |
| **EU (GDPR)** | Not literally required by GDPR text, but consent must be "specific, informed, unambiguous, freely given." Double opt-in is the practical way to **demonstrate** consent under Article 7(1). | [iubenda on GDPR DOI](https://www.iubenda.com/en/blog/gdpr-double-opt-in-2/) |
| **Germany** | **Effectively required.** German Federal Supreme Court (BGH) has ruled single opt-in is "by no means sufficient" to prove consent for email marketing. | [Demodia on German email law](https://demodia.com/articles/data-processes/is-double-opt-in-really-required-for-email-marketing-in-germany) |
| **Austria** | **Required.** The Austrian DPA has ruled missing double opt-in is itself a violation of GDPR Article 32 (security of processing). | [LinkedIn — Austrian DPA ruling](https://www.linkedin.com/pulse/austrian-data-protection-authority-missing-double-opt-in-piltz) |
| **Canada (CASL)** | Express consent is required. Double opt-in is not literally required, but is the easiest way to prove express consent (which is on you to prove). | [CRTC CASL guidance](https://crtc.gc.ca/eng/com500/guide.htm) |
| **UK (PECR)** | Single opt-in is sufficient for B2C as long as it's specific + unambiguous. Double opt-in is best practice but not mandated. | [ICO electronic marketing guide](https://ico.org.uk/for-organisations/direct-marketing-and-privacy-and-electronic-communications/guide-to-pecr/electronic-and-telephone-marketing/electronic-mail-marketing/) |

### 4.2 Tradeoffs

- **Single opt-in**: ~30–50% more confirmed subscribers, but you keep typos, bots, and spam-trap hits. List quality is materially worse.
- **Double opt-in**: Loses ~20–30% of signups (people who don't click the confirmation), but every address on the list has provably engaged at least once. Spam-trap risk drops to near zero. Complaint rate drops materially.

For a CLI built on Resend at the 10k–500k tier, **default should be double opt-in** because the cost of one bad import (pristine traps) is weeks of recovery. Single opt-in should be a non-default, opt-in flag with a warning.

### 4.3 Mechanical implementation

Standard pattern:

1. User submits email via signup form → CLI marks contact as `pending`, generates a single-use signed token (HMAC of `email + timestamp + secret`, OR random nonce stored in DB).
2. CLI sends a confirmation email (transactional, NOT a broadcast) with a `https://yourdomain/confirm?token=...` link.
3. Click → token validated → contact marked `confirmed`, timestamp + IP + user-agent + form-source recorded as the **consent record** (Article 7 evidence).
4. If not confirmed within 7 days → token expires; contact stays `pending` and never receives marketing mail.

The CLI must store the **consent proof bundle** for each subscriber: timestamp, IP, source URL, form fields shown, opt-in language, double-opt-in confirmation timestamp. This is the only way to defend yourself in a regulator complaint.

---

## 5. List Hygiene

### 5.1 Re-engagement / sunset campaigns

**When to run**: any subscriber with no open AND no click for 6 months on a monthly-cadence list, or 3 months on a weekly-cadence list ([Mailjet sunset policies](https://www.mailjet.com/blog/deliverability/understanding-email-sunset-policies/), [GetResponse sunset guide](https://www.getresponse.com/blog/sunset-policy)).

**Standard staged sunset**:

1. **Day 0–90 inactive**: still in main segment
2. **Day 90–180**: tag as `at_risk`, reduce send frequency, send a "we miss you" email
3. **Day 180**: trigger a 2–3 email re-engagement sequence over 4–6 weeks ("Are you still interested? Click here to stay subscribed")
4. **No engagement after re-engagement**: auto-suppress with reason `inactive_sunsetted`

### 5.2 Signals that justify auto-removal

- Hard bounce (immediate)
- 5+ consecutive soft bounces (immediate)
- Spam complaint (immediate)
- 12 months no opens AND no clicks (after sunset campaign fails)
- Spam-trap hit (immediate, plus investigate acquisition source)
- GDPR Article 17 erasure request (immediate, with hard delete)
- Subscribed > 6 months ago, never opened a single email (likely typo or bot)

The CLI should expose all of these as configurable rules and run them on a schedule.

---

## 6. Compliance — Minimum Legal Requirements

### 6.1 CAN-SPAM (US, 15 U.S.C. §7701 et seq.)

Per the [FTC CAN-SPAM compliance guide](https://www.ftc.gov/business-guidance/resources/can-spam-act-compliance-guide-business), every commercial email must:

1. **Truthful headers** — From, Reply-To, routing headers must accurately identify the sender.
2. **Non-deceptive subject line.**
3. **Clearly identify** the message as an ad (unless prior consent).
4. **Valid physical postal address** — current street address, USPS-registered PO box, or registered private mailbox.
5. **Clear and conspicuous unsubscribe** — visible, easy, no login required.
6. **Honor unsubscribes within 10 business days.**
7. **Opt-out mechanism must remain functional for at least 30 days after sending.**
8. **You're responsible even if a third party sends on your behalf.**

Penalty: up to **$53,088 per email** (FTC 2024 inflation-adjusted figure).

CLI implications:
- Every broadcast must support a `physical_address` field that gets injected into the footer. Refuse to send without one.
- Unsubscribe handling must be live within **2 days max** (Gmail/Yahoo rule) — well inside the 10-business-day CAN-SPAM ceiling.

### 6.2 GDPR (EU, Regulation 2016/679)

Per [GDPR Article 6](https://gdpr-info.eu/art-6-gdpr/) (lawful basis) and [Article 7](https://gdpr-info.eu/art-7-gdpr/) (consent conditions):

1. **Lawful basis required** — for marketing email, this is almost always (a) consent or (f) legitimate interest. For B2C, in practice, it must be consent.
2. **Consent must be**: freely given, specific, informed, unambiguous, by clear affirmative action. No pre-ticked boxes. No bundled consent.
3. **Right to withdraw consent** must be as easy as giving it (Article 7(3)).
4. **Records of consent** must be kept (Article 7(1)) — who, when, how, what they saw.
5. **Right to erasure** (Article 17) — must be honored "without undue delay", typically interpreted as 30 days.
6. **Right of access** (Article 15) — subscriber can ask what you have on them.
7. **Data portability** (Article 20) — must be exportable in machine-readable format.
8. **Breach notification** within 72 hours to the supervisory authority.

CLI implications:
- Must store consent proof bundle (see §4.3).
- Must have an `erase` command that performs **hard delete** of all PII for an email (not soft-delete in a "deleted" table — that's still processing).
- Must export a single subscriber's full record on demand.
- Must support **data subject access requests (DSARs)** without exposing other subscribers.

### 6.3 CASL (Canada, S.C. 2010, c. 23)

Per the [CRTC CASL guidance](https://crtc.gc.ca/eng/com500/guide.htm):

1. **Consent required** before sending a Commercial Electronic Message (CEM).
2. **Express consent** = explicit opt-in. Not time-limited until withdrawn.
3. **Implied consent** = e.g. existing business relationship (EBR). Time-limited:
   - Purchase or contract: **2 years** from the transaction.
   - Inquiry: **6 months** from the inquiry.
4. **Sender identification** must be clear (legal name, mailing address, contact method).
5. **Unsubscribe mechanism** must be functional for **60 days** after sending. Must process within **10 business days**.
6. **Burden of proof of consent is on the sender.**

Penalty: up to **CAD $10 million per violation** (corporate).

CLI implications:
- Must track consent type per subscriber (`express` vs `implied:purchase` vs `implied:inquiry`).
- For implied consent, must auto-expire after 2 years (purchase) or 6 months (inquiry) and either re-confirm or move to suppression.

### 6.4 PECR (UK, Privacy and Electronic Communications Regulations 2003)

Per the [ICO electronic marketing guide](https://ico.org.uk/for-organisations/direct-marketing-and-privacy-and-electronic-communications/guide-to-pecr/electronic-and-telephone-marketing/electronic-mail-marketing/), PECR sits **on top of UK GDPR** and is the operative law for marketing email to individuals in the UK:

- **Active opt-in required.** No pre-ticked boxes, no bundled consent.
- **Soft opt-in exception** (4 conditions, all required):
  1. You collected the address during a sale or sale negotiation.
  2. You're marketing only your own similar products/services.
  3. You gave a clear opt-out at collection time.
  4. You give a clear opt-out in every subsequent message.
- **Soft opt-in does NOT apply to**: prospective customers, purchased lists, charity fundraising, political campaigning.
- **B2B exemption**: corporate subscribers (e.g. `info@company.com`) are not "individuals" under PECR, so the consent rule doesn't apply — but UK GDPR still does, and you need a lawful basis.

CLI implication: track per-subscriber whether they're under `consent`, `soft_optin`, or `b2b_legitimate_interest`. Soft opt-in subscribers must have an opt-out in every send (which the One-Click List-Unsubscribe header provides automatically).

### 6.5 List-Unsubscribe header (RFC 8058) — is it mandatory now?

**Yes. Effectively mandatory since 1 June 2024 for any sender doing >5,000 messages/day to Gmail or Yahoo.** Even below that volume, it's a strong best practice and Gmail recommends it for all senders.

Per [RFC 8058](https://datatracker.ietf.org/doc/html/rfc8058) and [Gmail's bulk sender guidelines](https://support.google.com/a/answer/81126):

Every marketing/promotional email MUST include both headers:

```
List-Unsubscribe: <https://example.com/unsub?token=opaque_id>, <mailto:unsub@example.com?subject=unsub>
List-Unsubscribe-Post: List-Unsubscribe=One-Click
```

Constraints:
- The HTTPS URI must accept an HTTP **POST** with body `List-Unsubscribe=One-Click`.
- The POST endpoint **MUST NOT require cookies, auth headers, or any context** — it must work cold.
- Both headers must be **covered by a valid DKIM signature**.
- Unsubscribe must be honored within **2 days** (Gmail/Yahoo).
- The opaque token in the URI must be unguessable (otherwise bots will mass-unsubscribe your list).
- Transactional emails (password resets, receipts) are exempt.

Sources: [SocketLabs on 2024 mandate](https://www.socketlabs.com/blog/2024-is-the-year-of-the-one-click-list-unsubscribe/), [Mailgun on RFC 8058](https://www.mailgun.com/blog/deliverability/what-is-rfc-8058/).

CLI implications:
- The CLI must own the unsubscribe endpoint (or proxy to Resend's). Must POST-accept. Must work without any session.
- The CLI must inject these headers on every broadcast send. Refuse to send a broadcast if either header is missing.
- The CLI's confirmation/erasure endpoint must use signed tokens, not predictable IDs.

---

## 7. Sender Authentication & Warmup

### 7.1 SPF / DKIM / DMARC division of responsibility

| Standard | Purpose | Who configures |
|---|---|---|
| **SPF** ([RFC 7208](https://datatracker.ietf.org/doc/html/rfc7208)) | TXT record listing which IPs/hosts are allowed to send for the domain | **You**, on your DNS. Resend gives you the value to add. |
| **DKIM** ([RFC 6376](https://datatracker.ietf.org/doc/html/rfc6376)) | Cryptographic signature of message headers + body, verified via public key in DNS | **You** add the CNAME records Resend gives you. Resend signs the messages. |
| **DMARC** ([RFC 7489](https://datatracker.ietf.org/doc/html/rfc7489)) | Policy statement: what to do if SPF/DKIM fail; aggregate report endpoint | **You**, on your DNS. Resend can't set this for you because it lives at the apex. |

**Alignment** is the often-missed gotcha: SPF and DKIM can both pass while DMARC still fails, because DMARC requires the authenticated domain to **align** with the visible `From:` domain. Use Resend's DKIM CNAMEs at your own root domain (or a subdomain you actually use as `From:`) so alignment passes.

Recommended DMARC starting point:
```
v=DMARC1; p=none; rua=mailto:dmarc@yourdomain.com; pct=100; adkim=s; aspf=s
```
Then move `p=none → p=quarantine → p=reject` over weeks once aggregate reports show all legit traffic is aligned.

Sources: [SalesHive 2025 best practices](https://saleshive.com/blog/dkim-dmarc-spf-best-practices-email-security-deliverability/), [dmarcian alignment](https://dmarcian.com/alignment/).

### 7.2 IP / domain warmup

**When it matters**: only when you're using a dedicated IP. On Resend, that means [paid tier with sending volume >500/day](https://resend.com/docs/knowledge-base/how-do-dedicated-ips-work).

**Resend's behavior**: Managed Dedicated IP Pools handle warmup automatically. New IPs get traffic gradually distributed across the shared pool and the new dedicated pool. Volume ramps over 2–6 weeks ([Resend dedicated IPs](https://resend.com/blog/dedicated-ips)).

**Manual warmup schedule** (if you ever need to do it yourself, e.g. on a raw SES setup):

| Day | Max sends |
|---|---|
| 1 | 50 |
| 2 | 100 |
| 3 | 500 |
| 4 | 1,000 |
| 5 | 2,500 |
| 6 | 5,000 |
| 7 | 10,000 |
| 8–14 | double daily |
| 15+ | full volume |

Always start with your **most engaged** subscribers — recent opens / recent clicks. Never warm up by blasting the whole list.

Source: [SendGrid IP warmup guide](https://sendgrid.com/en-us/resource/email-guide-ip-warm-up), [Mailgun warmup schedule](https://www.mailgun.com/blog/deliverability/domain-warmup-reputation-stretch-before-you-send/).

**Critical**: the CLI should expose a "warmup mode" that caps daily sends and orders the queue by engagement score, so warmup is just `mailing-list-cli broadcast send --warmup`.

### 7.3 Gmail/Yahoo bulk sender rules (Feb–June 2024, enforced harder Nov 2025)

For senders of **>5,000 messages/day to Gmail or Yahoo**, all of the following are mandatory ([Gmail sender guidelines](https://support.google.com/a/answer/81126), [Resend on Gmail/Yahoo 2024 reqs](https://resend.com/blog/gmail-and-yahoo-bulk-sending-requirements-for-2024)):

1. **SPF** — set up and aligned.
2. **DKIM** — set up and aligned.
3. **DMARC** — published with at least `p=none`. Yahoo accepts `p=none` for now; some sources say Gmail will push for `p=quarantine` or `p=reject` over time.
4. **Forward-confirmed reverse DNS (FCrDNS)** — sending IP must have valid reverse DNS that resolves back to itself.
5. **TLS** for connections to Gmail/Yahoo.
6. **One-click unsubscribe** via RFC 8058 (`List-Unsubscribe` + `List-Unsubscribe-Post`) headers.
7. **Honor unsubscribes within 2 days.**
8. **Spam complaint rate < 0.30% in Postmaster Tools** (target < 0.10%).
9. **Format messages per RFC 5322** (proper structure, no malformed headers).

Below 5,000/day to Gmail you're not formally subject to all of these, but Gmail recommends them universally and ramped enforcement in Nov 2025. **Default to 100% compliance regardless of volume.** If your list is at 10k subscribers and you send a weekly newsletter, you're at 10k/week ≈ 1,400/day across all providers — but a single send to 10k recipients is 10k messages, which puts you over the daily threshold for that day.

---

## 8. Top 5 Things That Break at 10k+ That Don't at 1k

These are the silent killers. They don't break the API call. They break your reputation.

### 8.1 Per-campaign suppression instead of global suppression

**Root cause**: The system treats unsubscribes as belonging to a list/campaign, not to the sender. A subscriber unsubscribes from "Weekly Digest," then signs up to "Product News" through an embedded form, and now gets emails again. They mark it as spam.

**Symptom**: complaint rate climbs from 0.05% to 0.5% over 3–4 sends. Postmaster Tools turns red. Inbox placement collapses.

**Fix**: global suppression list, enforced at send time, no per-list opt-out without an "opt out of everything" option that updates the global list.

### 8.2 Hard-bounced addresses being re-imported

**Root cause**: A user re-uploads a CSV that includes addresses already in the suppression list as `hard_bounced`. The system imports them. Next campaign sends to them again.

**Symptom**: bounce rate jumps. Resend account-level reputation drops. Eventually Resend may suspend.

**Fix**: every import must filter against the global suppression list. Imports must be a no-op for any address in suppression. Surface the count of skipped addresses to the operator.

### 8.3 Slow / batched unsubscribe handling

**Root cause**: The system processes unsubscribes from a nightly cron job. A user unsubscribes Monday at 9am; the next campaign goes out Monday at 11am; they receive it again; they mark it as spam.

**Symptom**: complaint rate up. CAN-SPAM exposure (10 business days) is fine, but **Gmail's 2-day rule is not fine**, and at 10k subscribers your batches are big enough that even a few hours of lag matters.

**Fix**: unsubscribe webhook → suppression list, **synchronously**, before the next batch ever runs. The CLI should never send to an address that has unsubscribed since the previous batch was assembled — re-check at dispatch time.

### 8.4 Sending to the entire list every time, ignoring engagement

**Root cause**: At 1k subscribers, you can blast the whole list and Gmail mostly tolerates it. At 10k, Gmail starts measuring engagement. If your "open rate among subscribers who never opened anything from you" is low (which it is, by definition), Gmail throttles the rest.

**Symptom**: open rates collapse. Inbox-tab placement drops to Promotions, then Spam. Postmaster Tools shows degraded reputation.

**Fix**: segment by engagement (last open / last click), and send most campaigns to the engaged segment first. Send re-engagement-only campaigns to the unengaged. Sunset what won't re-engage.

### 8.5 Importing without consent provenance

**Root cause**: The operator imports a CSV from "an old list" or "a partner." The list contains pristine spam traps planted by Spamhaus, Validity, etc. First broadcast hits a trap on day one. Domain reputation tanks before you even know.

**Symptom**: first or second broadcast lands in Bulk/Spam at every major ISP. Reputation reports show "spam trap hit." Recovery takes weeks.

**Fix**: every imported row must carry a `consent_source`, `consent_timestamp`, `consent_ip`, `consent_form_url`. Imports without provenance must be rejected by default, or flagged with a giant warning that lays out the spam-trap risk. CLI should support a "dry run import" that runs the addresses through a verifier (Kickbox / NeverBounce / ZeroBounce) before any are added.

---

## Appendix: References

- [FTC — CAN-SPAM Act compliance guide](https://www.ftc.gov/business-guidance/resources/can-spam-act-compliance-guide-business)
- [GDPR Article 6 — lawfulness of processing](https://gdpr-info.eu/art-6-gdpr/)
- [GDPR Article 7 — conditions for consent](https://gdpr-info.eu/art-7-gdpr/)
- [GDPR Article 17 — right to erasure](https://gdpr-info.eu/art-17-gdpr/)
- [CRTC — Canada's Anti-Spam Legislation guidance](https://crtc.gc.ca/eng/com500/guide.htm)
- [ICO — UK PECR electronic mail marketing guide](https://ico.org.uk/for-organisations/direct-marketing-and-privacy-and-electronic-communications/guide-to-pecr/electronic-and-telephone-marketing/electronic-mail-marketing/)
- [RFC 8058 — Signaling One-Click Functionality for List Email Headers](https://datatracker.ietf.org/doc/html/rfc8058)
- [RFC 7489 — DMARC](https://datatracker.ietf.org/doc/html/rfc7489)
- [RFC 7208 — SPF](https://datatracker.ietf.org/doc/html/rfc7208)
- [RFC 6376 — DKIM](https://datatracker.ietf.org/doc/html/rfc6376)
- [RFC 3463 — Enhanced Mail System Status Codes](https://reviewmyemails.com/emailalmanac/industry-standards-and-best-practices/bounce-error-handling-standards/rfc-3463-bounce-classification-codes)
- [Gmail email sender guidelines](https://support.google.com/a/answer/81126)
- [Gmail email sender guidelines FAQ](https://support.google.com/a/answer/14229414)
- [Yahoo Sender Hub best practices](https://senders.yahooinc.com/best-practices/)
- [Resend — bounce handling docs](https://resend.com/docs/dashboard/emails/email-bounces)
- [Resend — webhooks introduction](https://resend.com/docs/webhooks/introduction)
- [Resend — Audiences docs](https://resend.com/docs/dashboard/audiences/introduction)
- [Resend — Managed Dedicated IPs](https://resend.com/blog/dedicated-ips)
- [Resend — Gmail/Yahoo 2024 requirements](https://resend.com/blog/gmail-and-yahoo-bulk-sending-requirements-for-2024)
- [Postmark — How to fix spam complaints](https://postmarkapp.com/blog/how-to-fix-email-spam-complaints)
- [Postmark — List-Unsubscribe header guide](https://postmarkapp.com/support/article/1299-how-to-include-a-list-unsubscribe-header)
- [SendGrid — IP warmup guide](https://sendgrid.com/en-us/resource/email-guide-ip-warm-up)
- [Mailgun — RFC 8058 explained](https://www.mailgun.com/blog/deliverability/what-is-rfc-8058/)
- [Mailgun — Domain warmup guide](https://www.mailgun.com/blog/deliverability/domain-warmup-reputation-stretch-before-you-send/)
- [SocketLabs — One-click List-Unsubscribe is now required](https://www.socketlabs.com/blog/2024-is-the-year-of-the-one-click-list-unsubscribe/)
- [Suped — Soft bounce suppression logic](https://www.suped.com/knowledge/email-deliverability/technical/what-is-the-recommended-soft-bounce-suppression-logic-for-email)
- [Suped — Recent Google bulk sender changes](https://www.suped.com/knowledge/email-deliverability/compliance/what-are-the-recent-changes-to-googles-bulk-sender-guidelines)
- [Litmus — Spam trap survival guide](https://www.litmus.com/blog/a-guide-to-spam-traps-and-how-to-avoid-them)
- [Mailjet — Sunset policies](https://www.mailjet.com/blog/deliverability/understanding-email-sunset-policies/)
- [Demodia — Double opt-in in Germany](https://demodia.com/articles/data-processes/is-double-opt-in-really-required-for-email-marketing-in-germany)
- [iubenda — Does GDPR require double opt-in?](https://www.iubenda.com/en/blog/gdpr-double-opt-in-2/)
- [Security Boulevard — 2025 Gmail/Yahoo enforcement update](https://securityboulevard.com/2025/11/google-and-yahoo-updated-email-authentication-requirements-for-2025/)
