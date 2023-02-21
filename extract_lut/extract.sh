#!/usr/bin/env bash

mangle_name() {
  local file=$1
  file=$(basename "$file")
  file=${file%.o}
  file=${file/-/_}
  file="$file.rs"
  echo "$file"
}

for i in "$1"/*.o; do
  name=$(mangle_name "$i")
  cargo run "$i" > "$name"
done
