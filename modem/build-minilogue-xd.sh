#!/usr/bin/env bash

name="modem"

set -euo pipefail

cargo build --release \
  --features logue_plugin \
  --bin logue_plugin \
  --target thumbv7em-none-eabihf

mkdir "$name"
$HOST_OBJCOPY -O binary ../target/thumbv7em-none-eabihf/release/logue_plugin "$name/payload.bin"
cp -a manifest.json "$name/"

zip -r -m -q "$name.mnlgxdunit" "$name/"
