#!/bin/sh
# Minimal stub of email-cli for tests. Returns canned JSON for agent-info and profile test.
case "$2" in
    "agent-info")
        echo '{"name":"email-cli","version":"0.4.0","commands":{}}'
        exit 0
        ;;
    "profile")
        if [ "$3" = "test" ]; then
            echo '{"version":"1","status":"success","data":{"reachable":true}}'
            exit 0
        fi
        ;;
esac
echo '{"version":"1","status":"error","error":{"code":"unsupported","message":"stub","suggestion":"this is a test stub"}}' >&2
exit 1
