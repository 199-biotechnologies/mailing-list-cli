# Session Handoff — Session 7 (v0.3.2 emergency hardening round 2)

**Date:** 2026-04-09
**Session:** v0.3.2 — GPT Pro full report processed, 3 more outage-class fixes + crates.io + Homebrew update. Successor to session 6 (v0.3.1).

---

## TL;DR

GPT Pro returned a 15-question, 35+ finding report. 4/4 v0.3.1 fixes validated. 3 more critical items identified and shipped as v0.3.2. The GPT Pro report is saved (inline in the user conversation — save to `docs/handoffs/2026-04-09-gpt-pro-hardening-review.md` in the next session for archival).

**Current state:**
- Branch: `main`, commit `f659820`, tag `v0.3.2`
- Tests: 131 unit + 65 integration = **196 passing** (was 189 at session start)
- crates.io: v0.3.2 published
- Homebrew: formula updated to v0.3.2 (SHA256 `db26e77a`)
- CI: in-flight at handoff, previous commits all green

---

## v0.3.2 fixes shipped

| # | Commit | Finding | Effect |
|---|---|---|---|
| 1 | `b2fcade` | F13.1: unsubscribe secret dev fallback | `pipeline.rs:190,383` `unwrap_or("mlc-unsubscribe-dev")` → `map_err(Config exit 2)`. Check moved before lock acquire so missing secret can't leave broadcast locked. |
| 2 | `7a27b32` | F9.1 (revised): dead profile field | Doc comment on `EmailCli::profile`. New `profile_list()` method. New `email_cli_single_profile` health check: 0→fail, 1→ok, >1→warn. Stub email-cli extended. `agent-info` gains `known_limitations` array. |
| 3 | `3d2681e` | F2.1: ESP-before-local-commit window | Migration 0005 `broadcast_send_attempt`. Write-ahead log: `prepared → esp_acked → applied \| failed`. Reconcile on resume from stored `applied_pairs`. Indeterminate `prepared` blocks with operator instructions. `broadcast_clear_lock_only` helper for early-error paths. SHA-256 of batch file for idempotent retry detection. F3.2 doc note on `historical_send_rates` + AGENTS.md. |
| 4 | `f659820` | Version bump + lock-clear fix for indeterminate + secret-check paths | Smoke-discovered that F13.1 secret check was after lock acquire, leaving broadcast locked. Moved pre-check before lock. Indeterminate path also clears lock before returning error. |

---

## Migration accounting (cumulative)

| Migration | Release |
|---|---|
| 0001_initial | pre-v0.2 |
| 0002_event_idempotency_and_kv | v0.2 |
| 0003_template_html_source | v0.2 (no-op) |
| 0004_broadcast_locks | v0.3.1 |
| 0005_broadcast_send_attempt | **v0.3.2** |
| 0006_content_snapshots_and_revenue | **v0.4 (planned)** |

---

## GPT Pro findings roadmap (for the v0.4/v0.5 plan author)

### Already addressed (v0.3.1 + v0.3.2)
F1.1, F2.1, F8.1 (partial), F9.1 (revised), F9.2, F13.1, plus F3.2 doc note.

### v0.5 candidates (correctness under overlap and crash)
F1.2 (read-modify-write races outside send loop), F2.2 (partial chunk = success bug), F2.3 (webhook handler not transactional), F3.1 (structured logging), F5.1 (contract violations in health/report/broadcast), F6.1 (error class collapse), F6.2 (error code catalog), F8.1 (full migration handling), F9.3 (strict response parsing), F12.1-12.3 (papercuts), F14.1 (doctrine tests).

### v0.6 candidates (forensics + audit + contract discipline)
F3.2 (fix event source properly), F4.1 (audit_log table), F5.2 (agent-info contract + idempotency), F7.1 (db/mod.rs split), F11.1 (chaos tests), F13.2 (file permissions), F15.1 (invocation_id).

### v0.7 candidates (scale)
F10.1 (streaming pipeline at 100k+).

---

## What to do next

### 1. The v0.4 plan is STILL not written
Session 5 decided the scope (16 items, 4 phases). Session 6 + 7 detoured into hardening. The v0.4 plan must use migration `0006`. Use `superpowers:writing-plans` to draft `docs/plans/2026-04-09-phase-11-v0.4-operator-superpowers.md` (phase 11, since v0.3.1 took phase 9, v0.3.2 took phase 10).

**The strategic tension (from the GPT Pro analysis summary):** GPT Pro flagged that shipping monetization (MON-1..6) on top of an unreliable event mirror and the partial-success-as-success bug means revenue attribution sits on foundations GPT Pro explicitly calls approximate. The user chose to proceed with v0.4 as planned (operator superpowers / monetization) rather than pivot to a full correctness release. The documented limitations in `agent-info → known_limitations` and `AGENTS.md` cover the honest framing. The correctness fixes go in v0.5+.

### 2. Archive the GPT Pro report
The full report is in the conversation (user message). Save it to `docs/handoffs/2026-04-09-gpt-pro-hardening-review.md` with a `Source: GPT-5 Pro, 2026-04-09` header.

### 3. Verify CI on v0.3.2 tag
```bash
gh run list --branch main --limit 3
```

---

## Test count history

| Version | Tests | Notes |
|---|---|---|
| v0.3.0 | 177 | production-grade 10k |
| v0.3.1 | 189 | +12: schema check, timeout, lock CAS, agent-info drift |
| **v0.3.2** | **196** | +7: F13.1 secret check, F9.1 multi-profile health, F2.1 attempt table (5 unit tests) |
| v0.4.0 target | ~230+ | |

---

*End of session 7 handoff.*
