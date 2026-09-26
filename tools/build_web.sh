#!/usr/bin/env bash
# Release web build staged in ./dist for Cloudflare (used by CI and deploy.sh).
# Fails on a degraded bundle: dx exits 0 even when wasm-opt or esbuild fail, shipping an unoptimised wasm
# or JS whose `./snippets/` imports 404 (blank page).
set -euo pipefail
cd "$(dirname "$0")/.."

target_dir=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')
out="$target_dir/dx/app/release/web/public"
# Dioxus CLI from cargo (binstall in CI); a bare `dx` may resolve to Deno's `dx` on Homebrew setups
dx_bin="${DX:-$HOME/.cargo/bin/dx}"

# dx never prunes old hashed files from its output; start clean so dist holds only this build
rm -rf "$out"
"$dx_bin" build --release --platform web 2>&1 | tee dx-build.log
if grep -q "ERROR" dx-build.log; then echo "::error::dx logged an ERROR (see above)"; exit 1; fi
if grep -l "snippets/" "$out"/assets/*.js; then echo "::error::unbundled wasm-bindgen snippet import"; exit 1; fi
rm dx-build.log

rm -rf dist
cp -R "$out" dist
cp deploy/_headers dist/_headers
echo "staged $(du -sh dist | cut -f1) in dist/"
