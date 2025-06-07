#!/usr/bin/env bash
# Builds the WASM version of the game into ./dist (open it through a local web server).
# Prerequisite: rustup target add wasm32-unknown-unknown
set -euo pipefail
cd "$(dirname "$0")/.."

# Must match the macroquad version in Cargo.lock
MACROQUAD_VERSION="v0.4.13"

cargo build --release --target wasm32-unknown-unknown

rm -rf dist
mkdir dist
cp target/wasm32-unknown-unknown/release/chasedow.wasm dist/
cp web/index.html dist/
# The game calls set_pc_assets_folder("assets"), which also applies on the web:
# files are fetched from assets/ next to the page
mkdir dist/assets
cp assets/player.png assets/background.ogg dist/assets/
curl -fsSL "https://raw.githubusercontent.com/not-fl3/macroquad/${MACROQUAD_VERSION}/js/mq_js_bundle.js" \
    -o dist/mq_js_bundle.js

echo "Web build ready in dist/ (try: basic-http-server dist)"
