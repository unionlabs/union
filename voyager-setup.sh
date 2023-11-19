nix run .#eth-devnet-deploy

nix run .#uniond -- tx wasm instantiate2 1 '{"default_timeout": 10000, "gov_contract": "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"}' "01" --label "relay" --gas=auto --gas-adjustment=1.4 -y --no-admin --from testkey --chain-id union-devnet-1 --keyring-backend test --home result

RUST_LOG=info,voyager=debug cargo run -p voyager -- -c $PWD/voyager-config.json setup bind-port --on ethereum-devnet --module-address 0xF8F7758FbcEfd546eAEff7dE24AFf666B6228e73 --port-id ucs01-relay

voy-send-msg '{"Sequence":[{"Aggregate":{"queue":[{"Lc":{"EthereumMinimal":{"Fetch":{"chain_id":"union-devnet-1","data":{"SelfClientState":{"at":"latest"}}}}}},{"Lc":{"EthereumMinimal":{"Fetch":{"chain_id":"union-devnet-1","data":{"SelfConsensusState":{"at":"latest"}}}}}}],"data":[],"receiver":{"CometblsMinimal":{"chain_id":"32382","data":{"CreateClient":{"config":{"cometbls_client_address":"0x83428c7db9815f482a39a1715684dcf755021997"}}}}}}},{"Aggregate":{"queue":[{"Lc":{"CometblsMinimal":{"Fetch":{"chain_id":"32382","data":{"SelfClientState":{"at":"latest"}}}}}},{"Lc":{"CometblsMinimal":{"Fetch":{"chain_id":"32382","data":{"SelfConsensusState":{"at":"latest"}}}}}}],"data":[],"receiver":{"EthereumMinimal":{"chain_id":"union-devnet-1","data":{"CreateClient":{"config":{"code_id":"0x80f37df3260a49d4f4193a24963b1753e1ecf3a09d67882be9ce9e4010ad0376"}}}}}}}]}'

# voy-send-msg '{"Lc":{"EthereumMinimal":{"Msg":{"chain_id":"union-devnet-1","data":{"ConnectionOpenInit":{"msg":{"client_id":"08-wasm-0","counterparty":{"client_id":"cometbls-0","connection_id":"","prefix":{"key_prefix":"0x696263"}},"version":{"identifier":"1","features":["Ordered","Unordered"]},"delay_period":0}}}}}}}'

# voy-send-msg '{"Lc":{"CometblsMinimal":{"Msg":{"chain_id":"32382","data":{"ChannelOpenInit":{"msg":{"port_id":"ucs01-relay","channel":{"state":"Init","ordering":"Unordered","counterparty":{"port_id":"wasm.union1qcmel8wqdwxn76mknvsquues6vetay458fd3nga5rlqnu4jru48saclnwg","channel_id":""},"connection_hops":["connection-0"],"version":"ucs01-0"}}}}}}}}'


