#!/usr/bin/env bash
# tests/support/stub_email_cli.sh
# A minimal email-cli stub for retry-behaviour tests.
#
# Modes (env-var driven):
#   STUB_EMAIL_CLI_FAIL_COUNT=N      -- fail the first N `batch send` calls
#                                        with a 429-shaped error, then succeed.
#                                        Counter is persisted to
#                                        STUB_EMAIL_CLI_COUNTER_FILE.
#   STUB_EMAIL_CLI_COUNTER_FILE=PATH -- where to stash the decrementing counter
#                                        across subprocess invocations.
#                                        Default: /tmp/stub-email-cli-counter
#   STUB_EMAIL_CLI_PERMANENT_4XX=1   -- always fail with a permanent 4xx-shaped
#                                        validation error (exit 3). No retries
#                                        should be attempted.
set -euo pipefail

# Strip the leading --json flag so the case statements work cleanly.
if [[ "${1:-}" == "--json" ]]; then
    shift
fi

case "$*" in
    "batch send"*|*"batch send"*)
        counter_file="${STUB_EMAIL_CLI_COUNTER_FILE:-/tmp/stub-email-cli-counter}"
        if [[ -n "${STUB_EMAIL_CLI_PERMANENT_4XX:-}" ]]; then
            echo '{"status":"error","error":{"code":"validation_error","message":"HTTP 422 Unprocessable Entity: invalid from address"}}' >&2
            exit 3
        fi
        if [[ -n "${STUB_EMAIL_CLI_FAIL_COUNT:-}" ]]; then
            current=$(cat "$counter_file" 2>/dev/null || echo "$STUB_EMAIL_CLI_FAIL_COUNT")
            if [[ "$current" -gt 0 ]]; then
                echo $((current - 1)) > "$counter_file"
                echo '{"status":"error","error":{"code":"rate_limited","message":"HTTP 429 Too Many Requests"}}' >&2
                exit 4
            fi
        fi
        echo '{"version":"1","status":"success","data":{"data":[{"id":"em_stub_1"}]}}'
        exit 0
        ;;
    "agent-info"*)
        echo '{"status":"success","data":{"version":"0.6.3-stub","profile":"stub"}}'
        exit 0
        ;;
    *)
        echo '{"status":"success","data":{}}'
        exit 0
        ;;
esac
