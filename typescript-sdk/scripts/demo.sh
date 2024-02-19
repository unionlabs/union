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
