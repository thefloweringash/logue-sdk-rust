#!/usr/bin/env bash

set -euo pipefail

name="$1"

set -euo pipefail

cargo build --release \
  --features wasm_module \
  --bin "${name}_wasm" \
  --target wasm32-unknown-unknown

cp "target/wasm32-unknown-unknown/release/${name}_wasm.wasm" "$name/$name.wasm"

sed -e "s/@MODULE_NAME@/$name/g" < test-template.html > "$name/test.html"
