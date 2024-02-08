#!/usr/bin/env bash

# jq 'map({"@type": "/cosmos.auth.v1beta1.BaseAccount", address: (.address), pub_key: null, account_number: 0, sequence: 0})' output.json

jq '
  map(.address) 
  | to_entries 
  | map(
    {
      "@type": "/cosmos.auth.v1beta1.BaseAccount", 
      address: (.value), 
      pub_key: null, 
      account_number: (.key), 
      sequence: 0
    }
  )
'
