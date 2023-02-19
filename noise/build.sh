#!/usr/bin/env bash

set -euo pipefail

cargo build --release

mkdir noise
$HOST_OBJCOPY -O binary ../target/thumbv7em-none-eabihf/release/noise noise/payload.bin
cp -a manifest.json noise/

zip -r -m -q noise.mnlgxdunit noise/
