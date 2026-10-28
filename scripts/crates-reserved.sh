#!/usr/bin/env bash
# Fails if a published workspace crate is not on crates.io yet. CI runs it in every PR.
#
# A new crate needs one first publish with an API token before release-plz can publish it
# through trusted publishing. This catches a missing one in its PR, not at release time.
#
# Usage: scripts/crates-reserved.sh, or `just crates-reserved`
set -euo pipefail
agent="senn crates-reserved (github.com/mrbandler/senn)"
missing=0
# `publish = false` shows up as an empty list; every other crate goes to crates.io.
for crate in $(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.publish != []) | .name'); do
  status="$(curl -sS -o /dev/null -w '%{http_code}' -A "$agent" "https://crates.io/api/v1/crates/$crate")"
  case "$status" in
    200) echo "$crate: reserved" ;;
    404) echo "$crate is not on crates.io yet. Run: just reserve-crate $crate" >&2; missing=1 ;;
    *) echo "crates.io: lookup of $crate failed with HTTP $status" >&2; exit 1 ;;
  esac
done
exit "$missing"
