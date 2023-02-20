#!/bin/bash
set -e
cd "`dirname $0`"
rustup target add wasm32-unknown-unknown
cargo build -p test_contract --target wasm32-unknown-unknown --release
cargo build -p test_token --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm ./res/