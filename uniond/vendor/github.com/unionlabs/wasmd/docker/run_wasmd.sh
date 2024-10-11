#!/bin/sh

if test -n "$1"; then
    # need -R not -r to copy hidden files
    cp -R "$1/.wasmd" /root
fi

mkdir -p /root/log
wasmd start --rpc.laddr tcp://0.0.0.0:26657 --trace
