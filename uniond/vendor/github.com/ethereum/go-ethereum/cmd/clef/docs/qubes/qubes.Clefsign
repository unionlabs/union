#!/bin/bash

SIGNER_BIN="/home/user/tools/clef/clef"
SIGNER_CMD="/home/user/tools/gtksigner/gtkui.py -s $SIGNER_BIN"

# Start clef if not already started
if [ ! -S /home/user/.clef/clef.ipc ]; then
	$SIGNER_CMD &
	sleep 1
fi

# Should be started by now
if [ -S /home/user/.clef/clef.ipc ]; then
    # Post incoming request to HTTP channel
	curl -H "Content-Type: application/json" -X POST -d @- http://localhost:8550 2>/dev/null
fi
