#!/bin/bash
# Uses valgrind's massif tool to compute heap memory consumption of compiled modules.
# For a wasmer `Modulej , it has been determined that this method underestimates the size
# of the module significanty.
# Use loupe::size_of_val instead, to get a better estimation.
set -e

MAX_SNAPSHOTS=1000

WASM="$1"
[ -z "$WASM" ] && echo "Usage: $0 <file>.wasm" && exit 1

PROFILE="release"
MEM_UTIL="valgrind --tool=massif --max-snapshots=$MAX_SNAPSHOTS"
SUM_UTIL="ms_print"

PROG=$(basename $0 .sh)
BASE_DIR=$(dirname $0)/..

# Look for the useful info
FNS="module_compile module_deserialize"

BIN="$BASE_DIR/../../target/$PROFILE/examples/$PROG"

RESULTS="$BASE_DIR/$PROG.log"
SUMMARY="$BASE_DIR/ms_print.log"

if [ "$PROFILE" = "release" ]; then
  RUSTFLAGS="-g" cargo build --release --example $PROG
else
  cargo build --example $PROG
fi

$MEM_UTIL --massif-out-file=$RESULTS $BIN $WASM
$SUM_UTIL $RESULTS >$SUMMARY

for FN in $FNS; do
  # Try to compute $FN() total (heap) bytes
  LAST_LINE=$(grep -n "::$FN " $SUMMARY | tail -1 | cut -f1 -d:)
  if [ -z "$LAST_LINE" ]; then
    echo -n "'$FN' not found. "
    [ $MAX_SNAPSHOTS -lt 1000 ] && echo "Try increasing MAX_SNAPSHOTS (current: $MAX_SNAPSHOTS, max: 1000). " || echo "Try again."
    continue
  fi
  TOTAL_LINES=$(wc -l $SUMMARY | cut -f1 -d\ )
  START_LINE=$((TOTAL_LINES - $LAST_LINE + 1))
  echo -n "module size ($FN): "
  tac $SUMMARY | sed -n "$START_LINE,/^  n /p" | grep "::$FN " | cut -f2 -d\( | cut -f1 -d\) | sort -u | sed 's/,//g;s/B//' | sed ':a;N;s/\n/+/;ta' | bc -l | sed 's/$/ bytes/'
done
