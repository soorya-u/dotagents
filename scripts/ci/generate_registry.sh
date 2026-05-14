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

    # Extract optional display name and URL from provider.toml.
    display_name=$(sed -n 's/^name = "\(.*\)"$/\1/p' "$d/provider.toml" | head -1)
    provider_url=$(sed -n 's/^url = "\(.*\)"$/\1/p' "$d/provider.toml" | head -1)

    jq \
      --arg name "$name" \
      --arg path "/v1/templates/$name/provider.toml" \
      --argjson checksums "$checksums" \
      --arg display_name "$display_name" \
      --arg provider_url "$provider_url" \
      '.providers[$name] = { "path": $path, "checksums": $checksums }
       | if $display_name != "" then .providers[$name].name = $display_name else . end
       | if $provider_url != "" then .providers[$name].url = $provider_url else . end' \
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
