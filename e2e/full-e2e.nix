{
  e2e,
  self',
  ...
}:
{
  all-works = e2e.mkE2eTestEthUnion {
    name = "all-works";

    testScript = ''
      devnetUnion.wait_until_succeeds("${self'.packages.cosmwasm-deployer}/bin/cosmwasm-deployer deploy-contract --rpc-url http://devnetUnion:26657 --private-key 0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f --bytecode ${self'.packages.lst} --init-msg '{ \"native_token_denom\": \"muno\", \"minimum_liquid_stake_amount\": \"10\", \"staker_address\": \"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2\", \"protocol_fee_config\": {\"fee_rate\": \"10000\", \"fee_recipient\": \"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2\" }, \"lst_address\": \"union1nluwd0qfymmdwfczezvgrmvz43n4xwdyfvshxj82sj7smuk9m42stgfwcz\", \"batch_period_seconds\": \"20\", \"monitors\": [], \"admin\": \"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2\", \"unbonding_period_seconds\": \"100\" }' --salt apps/lst --gas feemarket --max-gas 100000000 --gas-multiplier 1.4")

      devnetUnion.wait_until_succeeds("${self'.packages.cosmwasm-deployer}/bin/cosmwasm-deployer deploy-contract --rpc-url http://devnetUnion:26657 --private-key 0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f --bytecode ${self'.packages.cw-unionversal-token}  --init-msg '{ \"zkgm\": \"union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c\", \"admin\": \"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2\", \"cw20_init\": { \"cw20\": { \"name\": \"eU\", \"symbol\": \"eU\", \"decimals\": 6, \"initial_balances\": [], \"mint\": {\"minter\": \"union1qg3gm3f87w6al9u9ldkqhjdeaxrd0tae5w70les88egql8nzp95qs5rrz0\"} } }, \"extra_minters\": [] }' --salt tokens/eu --gas feemarket --max-gas 100000000 --gas-multiplier 1.4")

      devnetUnion.wait_until_succeeds("ls -la ${self'.packages.e2e-lst-tests}/lst")
      devnetUnion.wait_until_succeeds("RUST_LOG=info ${self'.packages.e2e-lst-tests}/lst --nocapture 1>2")
    '';

    nodes = {
    };
  };
}
