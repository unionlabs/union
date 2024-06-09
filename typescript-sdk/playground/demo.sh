#!/usr/bin/env bash

set -eou pipefail

npm run balance -- --chainId 11155111 --address 0xCa091fE8005596E64ba9Cf028a75755a2380021A

npm run balance -- --chainId 6 --address union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv

npm run transfer -- \
  --fromChainId 11155111 \
  --toChainId 6 \
  --fromPrivateKey 'enlist hip relief stomach skate base shallow young switch frequent cry park' \
  --toAddress union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv \
  --amount 99

npm run transfer -- \
  --fromChainId 6 \
  --toChainId 11155111 \
  --fromPrivateKey 'enlist hip relief stomach skate base shallow young switch frequent cry park' \
  --toAddress 0xCa091fE8005596E64ba9Cf028a75755a2380021A \
  --amount 100


cast send --rpc-url https://rpc2.sepolia.org \
    --private-key 0x1bdd5c2105f62c51d72c90d9e5ca6854a94337bcbcbb0b959846b85813d69380 \
    0xd0081080ae8493cf7340458eaf4412030df5feeb \
    "send(string, bytes, (address, uint128)[], (uint64, uint64), uint64)" "channel-22" "0xa833b03d8ed1228c4791cbfab22b3ed57954429f" "[(0x779877A7B0D9E8603169DdbD7836e478b4624789, 3)]" "(100, 1000000000)" "0"
