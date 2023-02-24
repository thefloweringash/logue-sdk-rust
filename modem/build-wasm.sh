#!/usr/bin/env bash

name="modem"

set -euo pipefail

cargo build --release \
  --features wasm_module \
  --bin wasm_module \
  --target wasm32-unknown-unknown

cp ../target/wasm32-unknown-unknown/release/wasm_module.wasm "./$name.wasm"
