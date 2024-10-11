#!/usr/bin/env bash

# travis_keepalive runs the given command and preserves its return value,
# while it forks a child process what periodically produces a log line,
# so that Travis won't abort the build after 10 minutes.

# Why?
# `t.Log()` in Go holds the buffer until the test does not pass or fail,
# and `-race` can increase the execution time by 2-20x.

set -euo pipefail

readonly KEEPALIVE_INTERVAL=300 # seconds => 5m

main() {
  keepalive
  $@
}

# Keepalive produces a log line in each KEEPALIVE_INTERVAL.
keepalive() {
  local child_pid
  # Note: We fork here!
  repeat "keepalive" &
  child_pid=$!
  ensureChildOnEXIT "${child_pid}"
}

repeat() {
  local this="$1"
  while true; do
    echo "${this}"
    sleep "${KEEPALIVE_INTERVAL}"
  done
}

# Ensures that the child gets killed on normal program exit.
ensureChildOnEXIT() {
  # Note: SIGINT and SIGTERM are forwarded to the child process by Bash
  # automatically, so we don't have to deal with signals.

  local child_pid="$1"
  trap "kill ${child_pid}" EXIT
}

main "$@"
