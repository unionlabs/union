#!/bin/bash

# Set up the routing needed for the simulation
/setup.sh

# The following variables are available for use:
# - ROLE contains the role of this execution context, client or server
# - SERVER_PARAMS contains user-supplied command line parameters
# - CLIENT_PARAMS contains user-supplied command line parameters

if [ "$ROLE" == "client" ]; then
        # Wait for the simulator to start up.
        /wait-for-it.sh sim:57832 -s -t 30
        ./interop -output=/downloads -qlog=$QLOGDIR $CLIENT_PARAMS $REQUESTS
elif [ "$ROLE" == "server" ]; then
        ./interop -cert=/certs/cert.pem -key=/certs/priv.key -qlog=$QLOGDIR -listen=:443 -root=/www "$@" $SERVER_PARAMS
fi
