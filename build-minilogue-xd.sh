#!/usr/bin/env bash

set -euo pipefail

name="$1"

src="$PWD/$name"
out="$PWD/$name/${name}.mnlgxdunit"

cargo build --release \
  --features logue_plugin \
  --bin "${name}_logue" \
  --target thumbv7em-none-eabihf

mkdir -p tmp
cd tmp

mkdir "$name"
$HOST_OBJCOPY -O binary "../target/thumbv7em-none-eabihf/release/${name}_logue" "$name/payload.bin"
cp -a "$src/manifest.json" "$name/"

zip -r -m -q "$out" "$name/"
