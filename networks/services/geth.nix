{ pkgs, config, genesis, ... }:
let
  geth-init =
    pkgs.writeShellApplication {
      name = "geth-init";
      runtimeInputs = [ pkgs.go-ethereum config genesis ];
      text = ''
        DATADIR=.
        ETH_DATADIR=$DATADIR/geth
        cp ${genesis}/genesis.json "$DATADIR/genesis.json"

        geth init --datadir "$ETH_DATADIR" "$DATADIR/genesis.json"
        geth account import --datadir "$ETH_DATADIR" \
          --password /dev/null ${config}/dev-key0.prv
        geth account import --datadir "$ETH_DATADIR" \
          --password /dev/null ${config}/dev-key1.prv

        geth \
          --http \
          --http.api=eth \
          --http.addr=0.0.0.0 \
          --authrpc.vhosts=* \
          --authrpc.addr=0.0.0.0 \
          --authrpc.jwtsecret=${config}/dev-jwt.prv \
          --datadir=$ETH_DATADIR \
          --allow-insecure-unlock \
          --unlock=0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD,0x89b4AB1eF20763630df9743ACF155865600daFF2 \
          --password=/dev/null \
          --nodiscover \
          --syncmode=full
      '';
    };
in
{
  image = {
    enableRecommendedContents = true;
    contents = [
      pkgs.coreutils
      pkgs.curl
      pkgs.jq
      geth-init
    ];
  };
  service = {
    stop_signal = "SIGINT";
    networks = [ "union-devnet" ];
    ports = [
      # Rest HTTP
      "8545:8545"
      # Auth RPC
      "8551:8551"
    ];
    command = [ "${geth-init}/bin/geth-init" ];
    healthcheck = {
      start_period = "20s";
      interval = "10s";
      retries = 4;
      test = [
        "CMD-SHELL"
        ''
          curl http://127.0.0.1:8545 \
            -X POST \
            -H 'Content-Type: application/json' \
            -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}' | jq -r '.result.hash' || exit 1
        ''
      ];
    };
  };
}
