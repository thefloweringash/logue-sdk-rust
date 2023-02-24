#!/usr/bin/env bash

set -euo pipefail

cargo build --release \
  --features logue_plugin \
  --bin logue_plugin \
  --target thumbv7em-none-eabihf

mkdir noise
$HOST_OBJCOPY -O binary ../target/thumbv7em-none-eabihf/release/logue_plugin noise/payload.bin
cp -a manifest.json noise/

zip -r -m -q noise.mnlgxdunit noise/
