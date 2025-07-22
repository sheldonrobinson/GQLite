#!/usr/bin/bash

BASE_PATH="crates/gqlitedb/pokec_iai/gqlitedb/pokec_iai/gqlitedb/pokec_iai/bench_micro_pokec_group"
TARGET_DIR=$1
CI_PIPELINE_ID=$2

for dir in "$BASE_PATH"/*; do
  if [ -d "$dir" ]; then
    SOMENAME=$(basename "$dir")
    SRC="$dir/summary.json"
    DST="${TARGET_DIR}/${CI_PIPELINE_ID}.${SOMENAME}.json"

    if [ -f "$SRC" ]; then
      echo "Moving $SRC to $DST"
      mv "$SRC" "$DST"
    else
      echo "Warning: $SRC not found"
    fi
  fi
done