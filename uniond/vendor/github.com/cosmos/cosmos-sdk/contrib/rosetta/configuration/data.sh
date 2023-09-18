#!/bin/sh

set -e

wait_simd() {
  timeout 30 sh -c 'until nc -z $0 $1; do sleep 1; done' localhost 9090
}
# this script is used to recreate the data dir
echo clearing /root/.simapp
rm -rf /root/.simapp/*
echo initting new chain
# init config files
simd init simd --chain-id testing

# create accounts
simd keys add fd --keyring-backend=test

addr=$(simd keys show fd -a --keyring-backend=test)
val_addr=$(simd keys show fd  --keyring-backend=test --bech val -a)

# give the accounts some money
simd genesis add-genesis-account "$addr" 1000000000000stake --keyring-backend=test

# save configs for the daemon
simd genesis gentx fd 10000000stake --chain-id testing --keyring-backend=test

# input genTx to the genesis file
simd genesis collect-gentxs
# verify genesis file is fine
simd genesis validate-genesis
echo changing network settings
sed -i 's/127.0.0.1/0.0.0.0/g' /root/.simapp/config/config.toml

# start simd
echo starting simd...
simd start --pruning=nothing &
pid=$!
echo simd started with PID $pid

echo awaiting for simd to be ready
wait_simd
echo simd is ready
sleep 10


# send transaction to deterministic address
echo sending transaction with addr $addr
simd tx bank send "$addr" cosmos19g9cm8ymzchq2qkcdv3zgqtwayj9asv3hjv5u5 100stake --yes --keyring-backend=test --chain-id=testing

sleep 10

echo stopping simd...
kill -9 $pid

echo zipping data dir and saving to /tmp/data.tar.gz

tar -czvf /tmp/data.tar.gz /root/.simapp

echo new address for bootstrap.json "$addr" "$val_addr"
