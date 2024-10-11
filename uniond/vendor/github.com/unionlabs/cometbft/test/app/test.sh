#! /bin/bash
set -ex

#- kvstore over socket, curl

# TODO: install everything

export PATH="$GOBIN:$PATH"
export CMTHOME=$HOME/.cometbft_app

function kvstore_over_socket(){
    rm -rf $CMTHOME
    cometbft init
    echo "Starting kvstore_over_socket"
    abci-cli kvstore > /dev/null &
    pid_kvstore=$!
    cometbft node > cometbft.log &
    pid_cometbft=$!
    sleep 5

    echo "running test"
    bash test/app/kvstore_test.sh "KVStore over Socket"

    kill -9 $pid_kvstore $pid_cometbft
}

# start cometbft first
function kvstore_over_socket_reorder(){
    rm -rf $CMTHOME
    cometbft init
    echo "Starting kvstore_over_socket_reorder (ie. start cometbft first)"
    cometbft node > cometbft.log &
    pid_cometbft=$!
    sleep 2
    abci-cli kvstore > /dev/null &
    pid_kvstore=$!
    sleep 5

    echo "running test"
    bash test/app/kvstore_test.sh "KVStore over Socket"

    kill -9 $pid_kvstore $pid_cometbft
}

case "$1" in 
    "kvstore_over_socket")
    kvstore_over_socket
    ;;
"kvstore_over_socket_reorder")
    kvstore_over_socket_reorder
    ;;
*)
    echo "Running all"
    kvstore_over_socket
    echo ""
    kvstore_over_socket_reorder
    echo ""
esac
