#!/usr/bin/env bash

set -euo pipefail

# Works in all cases except running from jar on Windows 
EXE=$APALACHE_HOME/bin/apalache-mc

CMD=${1:-typecheck}
N=${2:-10}
MC=${3:-true}
DD=${4:-false}

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
CFG=$SCRIPT_DIR/.apalache.cfg


if [[ $1 == "check" ]]; then
  FLAGS="--length=$N --inv=Inv --cinit=CInit --discard-disabled=$DD"
else
  FLAGS=""
fi

if ! [[ -f "$CFG" ]]; then
  echo "out-dir: \"$SCRIPT_DIR/_apalache-out\"" >> $CFG
  echo "write-intermediate: true" >> $CFG
fi

if [[ "$MC" = false ]]; then
  # Run 2c2f
  $EXE $CMD $FLAGS MC_PBT_2C_2F.tla  
else
  # Run 3c1f
  $EXE $CMD $FLAGS MC_PBT_3C_1F.tla  
fi



