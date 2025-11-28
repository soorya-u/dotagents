#!/usr/bin/env bash
set -e

if git diff --name-only HEAD~1 HEAD | grep -q "public/v1/templates/"; then
    echo "changes_detected=true" >> "$GITHUB_OUTPUT"
else
    echo "changes_detected=false" >> "$GITHUB_OUTPUT"
fi
