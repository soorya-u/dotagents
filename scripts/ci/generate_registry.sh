#!/usr/bin/env bash
set -e

ROOT="public/v1/templates"
REGISTRY="$ROOT/registry.json"
TMP_FILE="$ROOT/registry.tmp.json"

SCHEME_URL="https://dotagents.soorya-u.dev/v1/schemas/registry.schema.json"

jq -n \
  --arg scheme "$SCHEME_URL" \
  '
  {
    "$schema": $scheme,
    "providers": {}
  }
  ' > "$TMP_FILE"

# Fill providers from flat layout
for d in "$ROOT"/*; do
  if [ -d "$d" ] && [ -f "$d/provider.toml" ]; then
    name=$(basename "$d")
    jq \
      --arg name "$name" \
      --arg path "/templates/$name/provider.toml" \
      '.providers[$name] = { "path": $path }' \
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
