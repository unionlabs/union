{
  e2e,
  self',
  ...
}:
let
  voyagerConfigFile = ../tools/union-test/config.jsonc;
  voyagerNode = e2e.mkVoyagerNode voyagerConfigFile;
  voyagerBin = "${self'.packages.voyager}/bin/voyager";
in
{
  e2e-lst = e2e.mkE2eTestEthUnion voyagerConfigFile {
    name = "lst";

    openConnection = true;

    testScript = ''
      # open a channel since the lst tests require only 1 channel
      devnetVoyager.succeed(
        "echo '{\"@type\":\"call\",\"@value\":{\"@type\":\"submit_tx\",\"@value\":{\"chain_id\":\"32382\",\"datagrams\":[{\"ibc_spec_id\":\"ibc-union\",\"datagram\":{\"@type\":\"channel_open_init\",\"@value\":{\"counterparty_port_id\":\"0x756e696f6e3172667a33797467366c363077786b357278736b32376a766e32393037637961763034737a386b64653378686d6d66396e706c7871723879303563\",\"port_id\":\"0x05FD55C1AbE31D3ED09A76216cA8F0372f4B2eC5\",\"connection_id\":1,\"version\":\"ucs03-zkgm-0\"}}}]}}}' > /tmp/payload.json"
      )

      devnetVoyager.wait_until_succeeds("${voyagerBin} -c ${voyagerNode.voyagerConfig} q e $(cat /tmp/payload.json)")

      # wait until the channel is opened
      devnetVoyager.wait_until_succeeds("[[ $(${voyagerBin} rpc ibc-state 32382 '{ \"channel\": { \"channel_id\": 1 } }' | jq '.state.state == \"open\"') == true ]]")

      # deploy lst staker 
      devnetUnion.wait_until_succeeds("${self'.packages.cosmwasm-deployer}/bin/cosmwasm-deployer deploy-contract --rpc-url http://devnetUnion:26657 --private-key 0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f --bytecode ${self'.packages.lst-staker} --init-msg '{ \"local\": { \"admin\": \"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2\" } }' --salt apps/lst-staker --gas feemarket --max-gas 100000000 --gas-multiplier 1.4")

      # deploy lst
      devnetUnion.wait_until_succeeds("${self'.packages.cosmwasm-deployer}/bin/cosmwasm-deployer deploy-contract --rpc-url http://devnetUnion:26657 --private-key 0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f --bytecode ${self'.packages.lst} --init-msg '{ \"native_token_denom\": \"au\", \"minimum_liquid_stake_amount\": \"10\", \"staker_address\": \"union160a75a608j6w80x5ykckvd9cavs2xk8yfjzy2eqhpq0nprxg05qqf067nj\", \"protocol_fee_config\": {\"fee_rate\": \"10000\", \"fee_recipient\": \"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2\" }, \"lst_address\": \"union1nluwd0qfymmdwfczezvgrmvz43n4xwdyfvshxj82sj7smuk9m42stgfwcz\", \"batch_period_seconds\": \"20\", \"monitors\": [], \"admin\": \"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2\", \"unbonding_period_seconds\": \"100\" }' --salt apps/lst --gas feemarket --max-gas 100000000 --gas-multiplier 1.4")

      # deploy eU
      devnetUnion.wait_until_succeeds("${self'.packages.cosmwasm-deployer}/bin/cosmwasm-deployer deploy-contract --rpc-url http://devnetUnion:26657 --private-key 0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f --bytecode ${self'.packages.cw-unionversal-token}  --init-msg '{ \"zkgm\": \"union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c\", \"admin\": \"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2\", \"cw20_init\": { \"cw20\": { \"name\": \"eU\", \"symbol\": \"eU\", \"decimals\": 6, \"initial_balances\": [], \"mint\": {\"minter\": \"union1qg3gm3f87w6al9u9ldkqhjdeaxrd0tae5w70les88egql8nzp95qs5rrz0\"} } }, \"extra_minters\": [] }' --salt tokens/eu --gas feemarket --max-gas 100000000 --gas-multiplier 1.4")

      # set the fungible counterparty for U on union
      devnetUnion.wait_until_succeeds("\
        ${self'.packages.uniond}/bin/uniond tx \
          wasm execute \
          union1skg5244hpkad603zz77kdekzw6ffgpfrde3ldk8rpdz06n62k4hqct0w4j \
          '{\"set_fungible_counterparty\": {\"path\": \"0\", \"channel_id\": 1, \"base_token\": \"0x0c8C6f58156D10d18193A8fFdD853e1b9F8D8836\", \"counterparty_beneficiary\": \"0x0000000000000000000000000000000000000000000000000000000000000000\", \"escrowed_denom\": \"au\"}}' \
          --from alice \
          --gas auto \
          --gas-adjustment 10.0 \
          --node http://devnetUnion:26657 \
          --chain-id union-devnet-1 -y  \
          --home ${self'.packages.devnet-union-home} \
          --keyring-backend test \
          --gas-prices 1au\
        ")

      # set the validators on the staker
      devnetUnion.wait_until_succeeds("\
        ${self'.packages.uniond}/bin/uniond tx \
          wasm execute \
          union160a75a608j6w80x5ykckvd9cavs2xk8yfjzy2eqhpq0nprxg05qqf067nj \
          '{\"set_validators\": { \"unionvaloper1qp4uzhet2sd9mrs46kemse5dt9ncz4k3xuz7ej\": \"100\" }}' \
          --from alice \
          --gas auto \
          --gas-adjustment 10.0 \
          --node http://devnetUnion:26657 \
          --chain-id union-devnet-1 -y  \
          --home ${self'.packages.devnet-union-home} \
          --keyring-backend test \
          --gas-prices 1au\
        ")

      # set the lst hub address on staker
      devnetUnion.wait_until_succeeds("\
        ${self'.packages.uniond}/bin/uniond tx \
          wasm execute \
          union160a75a608j6w80x5ykckvd9cavs2xk8yfjzy2eqhpq0nprxg05qqf067nj \
          '{\"set_lst_hub_address\": \"union1qg3gm3f87w6al9u9ldkqhjdeaxrd0tae5w70les88egql8nzp95qs5rrz0\"}' \
          --from alice \
          --gas auto \
          --gas-adjustment 10.0 \
          --node http://devnetUnion:26657 \
          --chain-id union-devnet-1 -y  \
          --home ${self'.packages.devnet-union-home} \
          --keyring-backend test \
          --gas-prices 1au\
        ")

      # run the tests, note that we do `1>&2` because otherwise we won't get the full prints
      devnetUnion.wait_until_succeeds("RUST_LOG=info ${self'.packages.e2e-lst-tests}/lst --nocapture 1>&2")
    '';
  };
  e2e-zkgm = e2e.mkE2eTestEthUnion voyagerConfigFile {
    name = "zkgm";

    openConnection = true;

    testScript = ''
      # Deploy  .#cosmwasm-scripts.union-devnet.deploy here

      # run the tests, note that we do `1>&2` because otherwise we won't get the full prints
      devnetUnion.wait_until_succeeds("RUST_LOG=info ${self'.packages.e2e-lst-tests}/e2e --nocapture 1>&2")
    '';
  };
}
