{ pkgs, lodestar-cli, config }:
let
  lodestar-init = pkgs.writeShellApplication {
    name = "lodestar-init";
    runtimeInputs =
      [ pkgs.curl pkgs.jq lodestar-cli config ];
    text = ''
      ETH_ENDPOINT=http://geth:8545
      EXECUTION_ENDPOINT=http://geth:8551
      curl "$ETH_ENDPOINT" \
                        -X POST \
                        -H 'Content-Type: application/json' \
                        -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}'
      ETH_GENESIS_HASH=$(curl "$ETH_ENDPOINT" \
        -X POST \
        -H 'Content-Type: application/json' \
        -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}' | jq -r '.result.hash')
      GENESIS_TIMESTAMP=$(date -d'+10second' +%s)
      lodestar-cli dev \
        --genesisTime "$GENESIS_TIMESTAMP" \
        --genesisEth1Hash "$ETH_GENESIS_HASH" \
        --genesisValidators 8 \
        --startValidators "0..7" \
        --rest.address "0.0.0.0" \
        --rest.cors "*" \
        --eth1.providerUrls "$ETH_ENDPOINT" \
        --execution.urls "$EXECUTION_ENDPOINT" \
        --reset \
        --terminal-total-difficulty-override 0 \
        --params.ALTAIR_FORK_EPOCH 0 \
        --params.BELLATRIX_FORK_EPOCH 0 \
        --params.CAPELLA_FORK_EPOCH 0 \
        --eth1=true \
        --jwt-secret ${config}/dev-jwt.prv \
        --rest.namespace "*"
    '';
  };
in
{
  image = {
    enableRecommendedContents = true;
    contents = [
      pkgs.coreutils
      lodestar-init
    ];
  };
  service = {
    stop_signal = "SIGINT";
    networks = [ "union-devnet" ];
    ports = [
      # Beacon node rest API
      "9596:9596"
    ];
    command = [ "${lodestar-init}/bin/lodestar-init" ];
    depends_on = {
      geth = {
        condition = "service_healthy";
      };
    };
  };
}
