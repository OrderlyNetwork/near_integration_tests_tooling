#!/bin/bash
set -e
cd "`dirname $0`"
rustup target add wasm32-unknown-unknown
cargo build -p test_contract --target wasm32-unknown-unknown --release
cargo build -p test_token --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm ./res/
wasm-opt -O4 ./res/test_contract.wasm -o ./res/test_contract.wasm --strip-debug
wasm-opt -O4 ./res/test_token.wasm -o ./res/test_token.wasm --strip-debug