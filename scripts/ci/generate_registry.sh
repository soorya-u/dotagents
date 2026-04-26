#!/usr/bin/env bash
set -euo pipefail

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

    # Build checksums object: provider.toml + every .hbs file present in the dir.
    checksums="{}"
    files=("provider.toml")
    while IFS= read -r -d '' hbs; do
      files+=("$(basename "$hbs")")
    done < <(find "$d" -maxdepth 1 -type f -name '*.hbs' -print0 | sort -z)
    for f in "${files[@]}"; do
      [ -f "$d/$f" ] || continue
      checksum=$(sha256sum "$d/$f" | cut -d' ' -f1)
      checksums=$(echo "$checksums" | jq --arg file "$f" --arg sum "$checksum" '.[$file] = $sum')
    done

    jq \
      --arg name "$name" \
      --arg path "/templates/$name/provider.toml" \
      --argjson checksums "$checksums" \
      '.providers[$name] = { "path": $path, "checksums": $checksums }' \
      "$TMP_FILE" > "${TMP_FILE}.new"
    mv "${TMP_FILE}.new" "$TMP_FILE"
  fi
done

# Replace registry.json only if changed
if [ ! -f "$REGISTRY" ] || ! cmp -s "$TMP_FILE" "$REGISTRY"; then
  mv "$TMP_FILE" "$REGISTRY"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then echo "updated=true" >> "$GITHUB_OUTPUT"; fi
else
  rm "$TMP_FILE"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then echo "updated=false" >> "$GITHUB_OUTPUT"; fi
fi
