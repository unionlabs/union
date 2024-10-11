#!/bin/bash
set -eu

wasmd start --rpc.laddr tcp://0.0.0.0:26657 --log_level=info  --trace #remove trace flag if you don't wantg the stack trace to be printed
