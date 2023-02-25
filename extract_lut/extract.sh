#!/usr/bin/env bash

set -euo pipefail
shopt -s nullglob

mangle_name() {
  local file=$1
  file=$(basename "$file")
  file=${file%.o}
  file=${file/-/_}
  file="$file.rs"
  echo "$file"
}

cargo build --release
extract_lut=../target/release/extract_lut

rm -rf ../logue_sdk/src/lut
mkdir ../logue_sdk/src/lut

for i in "$1"/*.o; do
  name=$(mangle_name "$i")
  "$extract_lut" "$i" > "../logue_sdk/src/lut/$name"
done
