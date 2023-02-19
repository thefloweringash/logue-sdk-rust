#!/usr/bin/env bash

set -euo pipefail

mkdir noise

cargo build --release

$HOST_OBJCOPY -O binary ../target/thumbv7em-none-eabihf/release/noise noise/payload.bin
cp -a manifest.json noise/

zip -r -m -q noise.mnlgxdunit noise/
