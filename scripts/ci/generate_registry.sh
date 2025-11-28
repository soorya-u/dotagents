#!/usr/bin/env bash
set -e

ROOT="public/v1/templates"
REGISTRY="$ROOT/registry.json"
TMP_FILE="$ROOT/registry.tmp.json"

SCHEME_URL="https://dotagents.soorya-u.dev/v1/schemas/registry.schema.json"

jq -n \
  --arg scheme "$SCHEME_URL" \
  --arg root "$ROOT" \
  '
  {
    "$schema": $scheme,
    "providers": {
      "cli": {},
      "ide": {}
    }
  }
  ' > "$TMP_FILE"

# Fill CLI providers
for d in "$ROOT/cli"/*; do
  if [ -d "$d" ] && [ -f "$d/provider.toml" ]; then
    name=$(basename "$d")
    jq \
      --arg name "$name" \
      --arg path "/templates/cli/$name/provider.toml" \
      '.providers.cli[$name] = { "path": $path }' \
      "$TMP_FILE" > "${TMP_FILE}.new"
    mv "${TMP_FILE}.new" "$TMP_FILE"
  fi
done

# Fill IDE providers
for d in "$ROOT/ide"/*; do
  if [ -d "$d" ] && [ -f "$d/provider.toml" ]; then
    name=$(basename "$d")
    jq \
      --arg name "$name" \
      --arg path "/templates/ide/$name/provider.toml" \
      '.providers.ide[$name] = { "path": $path }' \
      "$TMP_FILE" > "${TMP_FILE}.new"
    mv "${TMP_FILE}.new" "$TMP_FILE"
  fi
done

# Replace registry.json only if changed
if [ ! -f "$REGISTRY" ] || ! cmp -s "$TMP_FILE" "$REGISTRY"; then
  mv "$TMP_FILE" "$REGISTRY"
  echo "updated=true" >> "$GITHUB_OUTPUT"
else
  rm "$TMP_FILE"
  echo "updated=false" >> "$GITHUB_OUTPUT"
fi
