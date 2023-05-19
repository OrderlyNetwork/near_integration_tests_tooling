#!/bin/bash
set -e
cd "`dirname $0`"
rustup target add wasm32-unknown-unknown
cargo build -p test_contract --target wasm32-unknown-unknown --release
cargo build -p test_token --target wasm32-unknown-unknown --release

TEST_TOKEN=target/wasm32-unknown-unknown/release/test_token.wasm
if [ -f "$TEST_TOKEN" ]; then
    cp $TEST_TOKEN ./test_token/wasm_target/
    wasm-opt -O4 ./test_token/wasm_target/test_token.wasm -o ./test_token/wasm_target/test_token.wasm --strip-debug
fi

mkdir -p res
cp target/wasm32-unknown-unknown/release/*.wasm ./res/

wasm-opt -O4 ./res/test_contract.wasm -o ./res/test_contract.wasm --strip-debug
wasm-opt -O4 ./res/test_token.wasm -o ./res/test_token.wasm --strip-debug