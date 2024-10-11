#!/bin/bash

function color() {
    # Usage: color "31;5" "string"
    # Some valid values for color:
    # - 5 blink, 1 strong, 4 underlined
    # - fg: 31 red,  32 green, 33 yellow, 34 blue, 35 purple, 36 cyan, 37 white
    # - bg: 40 black, 41 red, 44 blue, 45 purple
    printf '\033[%sm%s\033[0m\n' "$@"
}

system=""
case "$OSTYPE" in
darwin*) system="darwin" ;;
linux*) system="linux" ;;
msys*) system="windows" ;;
cygwin*) system="windows" ;;
*) exit 1 ;;
esac
readonly system

# Get locations of pb.go and pb.gw.go files.
findutil="find"
# On OSX `find` is not GNU find compatible, so require "findutils" package.
if [ "$system" == "darwin" ]; then
    if [[ ! -x "/usr/local/bin/gfind" && ! -x "/opt/homebrew/bin/gfind" ]]; then
        color 31 "Make sure that GNU 'findutils' package is installed: brew install findutils"
        exit 1
    else
        export findutil="gfind"  # skipcq: SH-2034
    fi
fi
