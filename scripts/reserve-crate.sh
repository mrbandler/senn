#!/usr/bin/env bash
# Reserves a new crate name on crates.io and hands it to trusted publishing.
#
# crates.io trusted publishing can't create a crate, so every new crate needs one first
# publish with an API token. This script does that with a placeholder 0.0.0, then creates
# the trusted publisher for release-plz and allows only trusted publishing from then on.
# Each step is skipped if it is already done, so the script is safe to run again.
#
# Token: CARGO_REGISTRY_TOKEN, or a hidden prompt. Scopes: publish-new and
# trusted-publishing, crate pattern `senn*`. It is never written to disk.
#
# Usage: scripts/reserve-crate.sh <crate>, or `just reserve-crate <crate>`
set -euo pipefail

crate="${1:?usage: scripts/reserve-crate.sh <crate>}"
api="https://crates.io/api/v1"
agent="senn reserve-crate (github.com/mrbandler/senn)"
# Where release-plz publishes from (.github/workflows/release-plz.yml).
owner="mrbandler"
repo="senn"
workflow="release-plz.yml"
environment="release"

for tool in cargo curl jq; do
  command -v "$tool" >/dev/null || { echo "$tool not found: run this inside the senn devenv shell" >&2; exit 1; }
done

token="${CARGO_REGISTRY_TOKEN:-}"
if [[ -z "$token" ]]; then
  [[ -t 0 ]] || { echo "no CARGO_REGISTRY_TOKEN and stdin is not a terminal, nothing changed" >&2; exit 1; }
  read -rsp "crates.io API token (hidden): " token
  echo
fi
[[ -n "$token" ]] || { echo "no token given, nothing changed" >&2; exit 1; }

# Calls the crates.io API: prints the body on success, the status and body on stderr on failure.
# The token goes to curl over stdin (--config -), so it never shows up in the process list.
call() {
  local method="$1" path="$2" body="${3:-}"
  local out status
  out="$(mktemp)"
  status="$(printf 'header = "Authorization: %s"\n' "$token" |
    curl -sS --config - -o "$out" -w '%{http_code}' -X "$method" -A "$agent" \
      -H "Content-Type: application/json" ${body:+--data "$body"} "$api$path")"
  if [[ "$status" == 2* ]]; then
    cat "$out"
    rm -f "$out"
  else
    echo "crates.io: $method $path failed with HTTP $status: $(cat "$out")" >&2
    rm -f "$out"
    return 1
  fi
}

# 1. Publish a placeholder 0.0.0 unless the crate exists.
status="$(curl -sS -o /dev/null -w '%{http_code}' -A "$agent" "$api/crates/$crate")"
if [[ "$status" == 404 ]]; then
  dir="$(mktemp -d)"
  trap 'rm -rf "$dir"' EXIT
  mkdir "$dir/src"
  cat >"$dir/Cargo.toml" <<TOML
[package]
name = "$crate"
version = "0.0.0"
edition = "2024"
license = "MIT"
description = "Part of senn. Placeholder: the first release follows from the repository."
repository = "https://github.com/$owner/$repo"
readme = "README.md"
TOML
  printf '//! Placeholder. See <https://github.com/%s/%s>.\n' "$owner" "$repo" >"$dir/src/lib.rs"
  printf '# %s\n\nPlaceholder, part of [senn](https://github.com/%s/%s). The first real release comes from that repository.\n' \
    "$crate" "$owner" "$repo" >"$dir/README.md"

  # The upload can't be undone, so it needs an answer from a person at a terminal.
  [[ -t 0 ]] || { echo "stdin is not a terminal: run this in your own terminal, nothing changed" >&2; exit 1; }
  read -rp "Publish the placeholder $crate 0.0.0 to crates.io? This can't be undone. [y/N] " answer
  [[ "$answer" == "y" ]] || { echo "aborted, nothing changed"; exit 0; }
  CARGO_REGISTRY_TOKEN="$token" cargo publish --manifest-path "$dir/Cargo.toml" --allow-dirty
elif [[ "$status" == 200 ]]; then
  echo "$crate exists on crates.io, no placeholder needed"
else
  echo "crates.io: lookup of $crate failed with HTTP $status" >&2
  exit 1
fi

# 2. Make sure our trusted publisher exists, and no other one.
configs="$(call GET "/trusted_publishing/github_configs?crate=$crate")"
ours="$(jq --arg o "$owner" --arg r "$repo" --arg w "$workflow" --arg e "$environment" \
  '[.github_configs[] | select(.repository_owner == $o and .repository_name == $r
    and .workflow_filename == $w and .environment == $e)] | length' <<<"$configs")"
total="$(jq '.github_configs | length' <<<"$configs")"
if ((total > ours)); then
  echo "$crate has trusted publishers that are not $owner/$repo, $workflow, environment $environment:" >&2
  jq '.github_configs' <<<"$configs" >&2
  echo "nothing changed; remove them on crates.io first" >&2
  exit 1
fi
if ((ours == 0)); then
  body="$(jq -n --arg c "$crate" --arg o "$owner" --arg r "$repo" --arg w "$workflow" --arg e "$environment" \
    '{github_config: {crate: $c, repository_owner: $o, repository_name: $r, workflow_filename: $w, environment: $e}}')"
  call POST "/trusted_publishing/github_configs" "$body" >/dev/null
  echo "trusted publisher created: $owner/$repo, $workflow, environment $environment"
else
  echo "$crate already has the trusted publisher $owner/$repo, $workflow, environment $environment"
fi

# 3. Allow only trusted publishing from now on.
call PATCH "/crates/$crate" '{"crate":{"trustpub_only":true}}' >/dev/null
echo "$crate accepts trusted publishing only: https://crates.io/crates/$crate"
