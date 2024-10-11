#!/bin/bash
#
# Boot a ping-pong network simulation using the HTTP API started by ping-pong.go

set -e

main() {
  if ! which p2psim &>/dev/null; then
    fail "missing p2psim binary (you need to build cmd/p2psim and put it in \$PATH)"
  fi

  info "creating 10 nodes"
  for i in $(seq 1 10); do
    p2psim node create --name "$(node_name $i)"
    p2psim node start "$(node_name $i)"
  done

  info "connecting node01 to all other nodes"
  for i in $(seq 2 10); do
    p2psim node connect "node01" "$(node_name $i)"
  done

  info "done"
}

node_name() {
  local num=$1
  echo "node$(printf '%02d' $num)"
}

info() {
  echo -e "\033[1;32m---> $(date +%H:%M:%S) ${@}\033[0m"
}

fail() {
  echo -e "\033[1;31mERROR: ${@}\033[0m" >&2
  exit 1
}

main "$@"
