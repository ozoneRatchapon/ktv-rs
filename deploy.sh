#!/usr/bin/env bash
# Production deploy: static web build -> Cloudflare Workers static assets (wrangler.jsonc).
# Gitflow: releases ship from `main` only, clean and pushed. No D1/KV/DO bindings, so nothing to back up first.
# `./deploy.sh --dry-run` builds and validates without uploading (any branch).
set -euo pipefail
cd "$(dirname "$0")"

WRANGLER="npx --yes wrangler@4.141.0"
dry_run=false
[[ "${1:-}" == "--dry-run" ]] && dry_run=true

if ! $dry_run; then
    branch=$(git rev-parse --abbrev-ref HEAD)
    [[ "$branch" == "main" ]] || { echo "deploy from main only (on $branch); use --dry-run to test"; exit 1; }
    [[ -z "$(git status --porcelain)" ]] || { echo "working tree not clean"; exit 1; }
    git fetch -q origin main
    [[ "$(git rev-parse HEAD)" == "$(git rev-parse origin/main)" ]] || { echo "HEAD differs from origin/main; push first"; exit 1; }
fi

./tools/build_web.sh

if $dry_run; then
    $WRANGLER deploy --dry-run
    exit 0
fi

version=$(git describe --tags --always)
$WRANGLER deploy --tag "$version" --message "$version $(git rev-parse --short HEAD)"
# Real prod state lives in Cloudflare, not git
$WRANGLER deployments list
